//! Téléchargement par batch
//!
//! Ce module gère le téléchargement progressif de grandes quantités
//! de données par batch pour éviter de surcharger l'API.

use iced::Task;
use std::sync::Arc;
use crate::finance_chart::core::SeriesId;
use crate::app::{
    messages::Message,
    app_state::ChartApp,
};

/// Charge l'historique complet d'une série depuis Binance
pub fn load_full_history(app: &mut ChartApp, series_id: SeriesId) -> Task<Message> {
    // Vérifier si le format est compatible avec Binance
    let series_name = if let Some(series) = app.chart_state.series_manager.get_series(&series_id) {
        let name = series.full_name();
        if !crate::app::realtime::realtime_utils::is_binance_format(&name) {
            println!("  ⚠️  {}: Format incompatible avec Binance (attendu: SYMBOL_INTERVAL)", name);
            return Task::none();
        }
        name
    } else {
        eprintln!("❌ Série {} introuvable", series_id.name);
        return Task::none();
    };
    
    println!("🔄 Chargement de l'historique complet pour {}...", series_name);
    
    // Arc::clone est très efficace (juste un compteur atomique)
    let provider = Arc::clone(&app.binance_provider);
    
    // Créer une Task async pour télécharger l'historique complet
    Task::perform(
        async move {
            let result = provider.fetch_full_history_async(&series_id)
                .await
                .map_err(|e| e.to_string());
            (series_id, series_name, result)
        },
        |(series_id, series_name, result)| Message::LoadFullHistoryComplete(series_id, series_name, result),
    )
}

/// Télécharge un batch de données et met à jour le graphique
/// Télécharge du plus récent vers le plus ancien (target_end -> current_start)
/// 
/// Note: fetch_candles_backwards_async ne spécifie pas de startTime, donc elle peut
/// retourner des bougies avant gap_start. On filtre ensuite pour ne garder que celles
/// dans le gap. Pour les très grands gaps, on pourrait utiliser fetch_all_candles_in_range_async
/// à la place pour plus d'efficacité.
pub fn download_batch(app: &mut ChartApp, series_id: &SeriesId) -> Task<Message> {
    let progress = match app.download_manager.get_progress(series_id) {
        Some(p) => p.clone(),
        None => {
            println!("  ⚠️ Pas de progress pour {}, arrêt du téléchargement", series_id.name);
            return Task::none();
        }
    };
    
    let provider = Arc::clone(&app.binance_provider);
    let series_id_clone = progress.series_id.clone();
    let gap_start = progress.current_start;  // timestamp le plus ancien du gap (objectif)
    let current_end = progress.target_end;     // timestamp actuel (on descend vers gap_start)
    let current_count = progress.current_count;
    let estimated_total = progress.estimated_total;
    
    println!("  🔄 Batch: de {} vers {} (objectif >= {})", current_end, gap_start, gap_start);
    
    Task::perform(
        async move {
            // Petite pause pour éviter de surcharger l'API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Télécharger les 1000 bougies les plus récentes avant current_end
            // Note: Cette fonction ne spécifie pas startTime, donc peut retourner des bougies
            // avant gap_start. On filtre ensuite.
            match provider.fetch_candles_backwards_async(&series_id_clone, gap_start, current_end).await {
                Ok(all_candles) => {
                    let raw_count = all_candles.len();
                    
                    if raw_count == 0 {
                        println!("    ⚠️ Batch vide, gap terminé");
                        return (series_id_clone.clone(), Vec::new(), current_count, estimated_total, gap_start);
                    }
                    
                    // L'API retourne les bougies triées par timestamp croissant (du plus ancien au plus récent)
                    // La première bougie est donc la plus ancienne du batch
                    let oldest_in_batch = all_candles.first().map(|c| c.timestamp).unwrap_or(current_end);
                    
                    // Filtrer pour ne garder que les bougies dans le gap (>= gap_start et <= current_end)
                    let filtered_candles: Vec<_> = all_candles
                        .into_iter()
                        .filter(|c| c.timestamp >= gap_start && c.timestamp <= current_end)
                        .collect();
                    
                    let filtered_count = filtered_candles.len();
                    let new_count = current_count + filtered_count;
                    
                    if filtered_count < raw_count {
                        println!("    📦 Batch: {} brutes, {} dans le gap (filtrage: {} exclues, oldest={})", 
                            raw_count, filtered_count, raw_count - filtered_count, oldest_in_batch);
                    } else {
                        println!("    📦 Batch: {} bougies dans le gap (oldest={})", 
                            filtered_count, oldest_in_batch);
                    }
                    
                    // Calculer le prochain end pour continuer le téléchargement
                    let next_end = if oldest_in_batch <= gap_start || raw_count < 1000 {
                        // On a atteint ou dépassé le début du gap, ou l'API n'a plus de données
                        println!("    ✅ Gap terminé (oldest={}, gap_start={})", oldest_in_batch, gap_start);
                        gap_start
                    } else {
                        // Continuer vers le passé: utiliser la bougie la plus ancienne du batch - 1
                        oldest_in_batch - 1
                    };
                    
                    (series_id_clone.clone(), filtered_candles, new_count, estimated_total, next_end)
                }
                Err(e) => {
                    eprintln!("  ❌ Erreur téléchargement: {}", e);
                    (series_id_clone.clone(), Vec::new(), current_count, estimated_total, gap_start)
                }
            }
        },
        move |(series_id, candles, count, estimated, next_end)| {
            Message::BatchDownloadResult(series_id, candles, count, estimated, next_end)
        },
    )
}







