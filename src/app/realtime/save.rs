//! Sauvegarde asynchrone des séries
//!
//! Ce module gère la sauvegarde asynchrone des séries mises à jour
//! pour ne pas bloquer l'interface utilisateur.

use iced::Task;
use std::collections::HashSet;
use crate::finance_chart::core::SeriesId;
use crate::app::{
    messages::Message,
    app_state::ChartApp,
};

/// Sauvegarde les séries de manière asynchrone
pub fn save_series_async(app: &mut ChartApp, updated_series: HashSet<SeriesId>) -> Task<Message> {
    println!("💾 Lancement de la sauvegarde asynchrone des séries mises à jour...");
    
    // Collecter les données à sauvegarder (cloner ce qui est nécessaire)
    let save_requests: Vec<(String, String, String, Vec<crate::finance_chart::core::Candle>, std::path::PathBuf)> = updated_series
        .iter()
        .filter_map(|series_id| {
            app.chart_state.series_manager.get_series(series_id)
                .map(|series| {
                    // Utiliser la nouvelle structure: data/Binance/{Symbol}/{interval}.json
                    // Utiliser le nouveau format de nommage: 1min.json pour 1m, 1month.json pour 1M
                    use std::path::PathBuf;
                    use crate::finance_chart::data_loader::interval_to_filename;
                    let data_dir = PathBuf::from("data");
                    let provider_dir = data_dir.join("Binance");
                    let symbol_dir = provider_dir.join(&series.symbol);
                    let file_name = interval_to_filename(&series.interval);
                    let file_path = symbol_dir.join(&file_name);
                    let file_path_str = file_path.to_string_lossy().to_string();
                    
                    let symbol = series.symbol.clone();
                    let interval = series.interval.clone();
                    // Cloner toutes les bougies
                    let candles: Vec<crate::finance_chart::core::Candle> = series.data.all_candles().to_vec();
                    (file_path_str, symbol, interval, candles, file_path)
                })
        })
        .collect();
    
    if save_requests.is_empty() {
        return Task::none();
    }
    
    // Lancer la sauvegarde dans un thread dédié
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer les dossiers si nécessaire
            use std::fs;
            for (_, _, _, _, ref file_path) in &save_requests {
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            
            let futures: Vec<_> = save_requests
                .into_iter()
                .map(|(file_path, symbol, interval, candles, _file_path_buf)| {
                    let file_path_clone = file_path.clone();
                    async move {
                        // Extraire le nom de la série depuis le chemin du fichier
                        let series_name = std::path::Path::new(&file_path_clone)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or_else(|| {
                                // Fallback: utiliser le nom du fichier sans extension
                                file_path_clone
                                    .trim_start_matches("data/")
                                    .trim_end_matches(".json")
                            })
                            .to_string();
                        
                        // Cloner series_name pour l'utiliser après le spawn_blocking
                        let series_name_for_result = series_name.clone();
                        
                        let result = tokio::task::spawn_blocking(move || {
                            // Utiliser save_to_json en créant une SeriesData temporaire
                            use crate::finance_chart::{core::{SeriesData, SeriesId, TimeSeries}, data_loader::save_to_json};
                            
                            // Utiliser le nom de la série (pas le chemin complet)
                            let series_id = SeriesId::new(series_name);
                            let timeseries = {
                                let mut ts = TimeSeries::new();
                                let mut errors = Vec::new();
                                for (idx, candle) in candles.iter().enumerate() {
                                    if let Err(e) = ts.push(candle.clone()) {
                                        errors.push(format!("Bougie {}: {}", idx, e));
                                    }
                                }
                                if !errors.is_empty() {
                                    eprintln!("⚠️ Erreurs lors de la reconstruction du TimeSeries:");
                                    for err in &errors {
                                        eprintln!("  - {}", err);
                                    }
                                }
                                ts
                            };
                            let series_data = SeriesData::new(series_id, symbol, interval, timeseries);
                            
                            save_to_json(&series_data, &file_path_clone)
                                .map_err(|e| e.to_string())
                        }).await;
                        
                        match result {
                            Ok(Ok(())) => (series_name_for_result, Ok(())),
                            Ok(Err(e)) => (series_name_for_result, Err(e)),
                            Err(e) => (series_name_for_result, Err(format!("Erreur de thread: {}", e))),
                        }
                    }
                })
                .collect();
            
            let results = join_all(futures).await;
            results
        },
        Message::SaveSeriesComplete,
    )
}






