//! Gestion du chargement asynchrone des séries de données
//!
//! Ce module gère le chargement des séries depuis les fichiers JSON de manière asynchrone
//! pour ne pas bloquer l'interface utilisateur au démarrage.

use iced::Task;
use crate::finance_chart::{
    load_all_from_directory, load_from_json, is_directory_empty, save_to_json,
    BinanceProvider, core::{SeriesId, TimeSeries, SeriesData}
};
use crate::app::{utils::constants::DATA_FILE, messages::Message};
use std::sync::Arc;

/// Intervalles disponibles pour Binance
const BINANCE_INTERVALS: &[&str] = &[
    "1m", "3m", "5m", "15m", "30m",
    "1h", "2h", "4h", "6h", "8h", "12h",
    "1d", "3d",
    "1w",
    "1M",
];

/// Télécharge uniquement les séries 1M (1 mois) pour un symbole donné depuis Binance
/// et crée les fichiers JSON dans data/Binance/{symbol}/1M.json
async fn download_1month_series_for_symbol(
    provider: Arc<BinanceProvider>,
    symbol: &str,
) -> Result<Vec<SeriesData>, String> {
    use std::fs;
    use std::path::PathBuf;
    
    let symbol_upper = symbol.to_uppercase();
    let data_dir = PathBuf::from("data");
    let provider_dir = data_dir.join("Binance");
    let symbol_dir = provider_dir.join(&symbol_upper);
    
    // Créer les dossiers si nécessaire
    fs::create_dir_all(&symbol_dir)
        .map_err(|e| format!("Erreur création dossier {}: {}", symbol_dir.display(), e))?;
    
    let mut downloaded_series = Vec::new();
    let interval = "1M";
    
    println!("🔄 Téléchargement de la série 1M pour {}...", symbol_upper);
    
    let series_id = SeriesId::new(format!("{}_{}", symbol_upper, interval));
    
    // Télécharger toutes les bougies pour l'intervalle 1M
    match provider.fetch_all_candles_async(&series_id).await {
        Ok(candles) => {
            if candles.is_empty() {
                println!("  ⚠️ Aucune bougie disponible pour {}_{}", symbol_upper, interval);
                return Ok(downloaded_series);
            }
            
            // Créer une TimeSeries à partir des bougies
            let mut timeseries = TimeSeries::new();
            for candle in candles {
                if let Err(e) = timeseries.push(candle) {
                    eprintln!("  ⚠️ Bougie invalide ignorée: {}", e);
                }
            }
            
            // Créer SeriesData
            let series = SeriesData::new(
                series_id.clone(),
                symbol_upper.clone(),
                interval.to_string(),
                timeseries,
            );
            
            // Sauvegarder dans le fichier JSON avec le nouveau format de nommage
            use crate::finance_chart::data_loader::interval_to_filename;
            let file_name = interval_to_filename(interval);
            let file_path = symbol_dir.join(&file_name);
            
            match save_to_json(&series, &file_path) {
                Ok(()) => {
                    println!("  ✅ {}: {} bougies sauvegardées", file_name, series.data.len());
                    downloaded_series.push(series);
                }
                Err(e) => {
                    eprintln!("  ❌ Erreur sauvegarde {}: {}", file_name, e);
                }
            }
        }
        Err(e) => {
            eprintln!("  ❌ Erreur téléchargement {}_{}: {}", symbol_upper, interval, e);
        }
    }
    
    println!("✅ Téléchargement 1M terminé: {} série(s) créée(s)", downloaded_series.len());
    Ok(downloaded_series)
}

/// Télécharge toutes les séries pour un symbole donné depuis Binance
/// et crée les fichiers JSON dans data/Binance/{symbol}/{interval}.json
async fn download_all_series_for_symbol(
    provider: Arc<BinanceProvider>,
    symbol: &str,
) -> Result<Vec<SeriesData>, String> {
    use std::fs;
    use std::path::PathBuf;
    
    let symbol_upper = symbol.to_uppercase();
    let data_dir = PathBuf::from("data");
    let provider_dir = data_dir.join("Binance");
    let symbol_dir = provider_dir.join(&symbol_upper);
    
    // Créer les dossiers si nécessaire
    fs::create_dir_all(&symbol_dir)
        .map_err(|e| format!("Erreur création dossier {}: {}", symbol_dir.display(), e))?;
    
    let mut downloaded_series = Vec::new();
    
    println!("🔄 Téléchargement des séries pour {}...", symbol_upper);
    
    for interval in BINANCE_INTERVALS {
        let series_id = SeriesId::new(format!("{}_{}", symbol_upper, interval));
        
        println!("  📥 Téléchargement {}_{}...", symbol_upper, interval);
        
        // Télécharger toutes les bougies pour cet intervalle
        match provider.fetch_all_candles_async(&series_id).await {
            Ok(candles) => {
                if candles.is_empty() {
                    println!("  ⚠️ Aucune bougie disponible pour {}_{}", symbol_upper, interval);
                    continue;
                }
                
                // Créer une TimeSeries à partir des bougies
                let mut timeseries = TimeSeries::new();
                for candle in candles {
                    if let Err(e) = timeseries.push(candle) {
                        eprintln!("  ⚠️ Bougie invalide ignorée: {}", e);
                    }
                }
                
                // Créer SeriesData
                let series = SeriesData::new(
                    series_id.clone(),
                    symbol_upper.clone(),
                    interval.to_string(),
                    timeseries,
                );
                
                // Sauvegarder dans le fichier JSON avec le nouveau format de nommage
                use crate::finance_chart::data_loader::interval_to_filename;
                let file_name = interval_to_filename(interval);
                let file_path = symbol_dir.join(&file_name);
                
                match save_to_json(&series, &file_path) {
                    Ok(()) => {
                        println!("  ✅ {}: {} bougies sauvegardées", file_name, series.data.len());
                        downloaded_series.push(series);
                    }
                    Err(e) => {
                        eprintln!("  ❌ Erreur sauvegarde {}: {}", file_name, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  ❌ Erreur téléchargement {}_{}: {}", symbol_upper, interval, e);
            }
        }
    }
    
    println!("✅ Téléchargement terminé: {} série(s) créée(s)", downloaded_series.len());
    Ok(downloaded_series)
}

/// Vérifie si une série 1M existe pour un symbole donné
fn check_1month_series_exists(symbol: &str) -> bool {
    use std::path::PathBuf;
    use crate::finance_chart::data_loader::interval_to_filename;
    let data_dir = PathBuf::from("data");
    let provider_dir = data_dir.join("Binance");
    let symbol_dir = provider_dir.join(symbol.to_string());
    // Vérifier les deux formats (ancien et nouveau)
    let file_path_new = symbol_dir.join(interval_to_filename("1M"));
    let file_path_old = symbol_dir.join("1M.json");
    file_path_new.exists() || file_path_old.exists()
}

/// Crée une Task pour charger les séries de manière asynchrone
/// Si le dossier data est vide, télécharge automatiquement les séries BTCUSDT
/// Sinon, vérifie et télécharge les séries 1M si elles n'existent pas
pub fn create_load_series_task(provider: Arc<BinanceProvider>) -> Task<Message> {
    Task::perform(
        async move {
            // Vérifier si le dossier data est vide dans un thread dédié
            let is_empty = tokio::task::spawn_blocking(|| {
                is_directory_empty("data")
            })
            .await
            .unwrap_or_else(|e| {
                eprintln!("❌ Erreur vérification dossier data: {}", e);
                Ok(false)
            })
            .unwrap_or(false);
            
            // Si le dossier est vide, télécharger les séries BTCUSDT
            if is_empty {
                println!("📂 Le dossier data est vide. Téléchargement automatique des séries BTCUSDT...");
                
                match download_all_series_for_symbol(provider, "BTCUSDT").await {
                    Ok(series_list) => {
                        println!("✅ {} série(s) téléchargée(s) et sauvegardée(s)", series_list.len());
                        // Recharger depuis le dossier maintenant qu'il contient des fichiers
                        tokio::task::spawn_blocking(move || {
                            match load_all_from_directory("data") {
                                Ok(series_list) => {
                                    println!("✅ {} série(s) chargée(s) depuis le dossier data", series_list.len());
                                    Ok(series_list)
                                }
                                Err(e) => {
                                    eprintln!("❌ Erreur lors du chargement des séries depuis 'data': {}", e);
                                    Err(format!("Erreur: {}", e))
                                }
                            }
                        })
                        .await
                        .unwrap_or_else(|e| Err(format!("Erreur de thread: {}", e)))
                    }
                    Err(e) => {
                        eprintln!("❌ Erreur lors du téléchargement des séries: {}", e);
                        Err(format!("Erreur téléchargement: {}", e))
                    }
                }
            } else {
                // Le dossier n'est pas vide, charger normalement
                let series_list_result: Result<Vec<SeriesData>, String> = tokio::task::spawn_blocking(move || {
                    load_all_from_directory("data")
                        .map_err(|e| format!("Erreur: {}", e))
                })
                .await
                .unwrap_or_else(|e| Err(format!("Erreur de thread: {}", e)));
                
                match series_list_result {
                    Ok(series_list) => {
                        println!("✅ {} série(s) trouvée(s) dans le dossier data", series_list.len());
                        
                        // Vérifier si les séries 1M existent pour les symboles chargés
                        let symbols: Vec<String> = series_list.iter()
                            .map(|s| s.symbol.clone())
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter()
                            .collect();
                        
                        // Télécharger les séries 1M manquantes
                        let provider_clone = Arc::clone(&provider);
                        for symbol in symbols {
                            let symbol_clone = symbol.clone();
                            let exists = tokio::task::spawn_blocking(move || {
                                check_1month_series_exists(&symbol_clone)
                            })
                            .await
                            .unwrap_or(false);
                            
                            if !exists {
                                println!("📥 Série 1M manquante pour {}, téléchargement...", symbol);
                                let provider_for_download = Arc::clone(&provider_clone);
                                if let Ok(_new_series) = download_1month_series_for_symbol(provider_for_download, &symbol).await {
                                    // Les nouvelles séries seront chargées au prochain démarrage
                                    println!("✅ Série 1M téléchargée pour {}", symbol);
                                }
                            }
                        }
                        
                        Ok(series_list)
                    }
                    Err(e) => {
                        eprintln!("❌ Erreur lors du chargement des séries depuis 'data': {}", e);
                        eprintln!("   Tentative de chargement du fichier par défaut: {}", DATA_FILE);
                        // Fallback: essayer de charger le fichier par défaut
                        tokio::task::spawn_blocking(move || {
                            match load_from_json(DATA_FILE) {
                                Ok(series) => {
                                    println!("✅ Série chargée: {} bougies", series.data.len());
                                    Ok(vec![series])
                                }
                                Err(e2) => {
                                    eprintln!("❌ Erreur de chargement: {}", e2);
                                    Err(format!("Erreur: {}", e2))
                                }
                            }
                        })
                        .await
                        .unwrap_or_else(|e| Err(format!("Erreur de thread: {}", e)))
                    }
                }
            }
        },
        Message::LoadSeriesFromDirectoryComplete,
    )
}

