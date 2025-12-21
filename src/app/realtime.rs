//! Gestion du temps réel et de la complétion des données
//!
//! Ce module gère toutes les opérations asynchrones liées à la mise à jour
//! en temps réel des données et à la complétion des gaps.

use iced::Task;
use std::collections::HashSet;
use crate::finance_chart::{
    UpdateResult,
    core::{SeriesId, Candle},
};
use crate::app::{
    messages::Message,
    utils::{interval_to_seconds, calculate_candles_back_timestamp},
    app_state::ChartApp,
};

/// Vérifie si le nom de série est au format Binance (SYMBOL_INTERVAL)
fn is_binance_format(series_name: &str) -> bool {
    // Validation stricte: doit contenir exactement un underscore
    // et avoir des parties non vides de chaque côté
    let parts: Vec<&str> = series_name.split('_').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
}

/// Complète les données manquantes pour toutes les séries
pub fn complete_missing_data(app: &mut ChartApp) -> Task<Message> {
    println!("🔄 Complétion des données manquantes depuis Binance...");
    
    // Collecter toutes les informations nécessaires d'abord
    let mut updates: Vec<(SeriesId, String, Option<i64>)> = Vec::new();
    
    for series in app.chart_state.series_manager.all_series() {
        let series_id = series.id.clone();
        let series_name = series.full_name();
        
        // Vérifier si le format est compatible avec Binance (SYMBOL_INTERVAL)
        if !is_binance_format(&series_name) {
            println!("  ⚠️  {}: Format incompatible avec Binance (attendu: SYMBOL_INTERVAL)", series_name);
            continue;
        }
        
        // Récupérer le dernier timestamp connu
        let last_ts = series.data.max_timestamp();
        updates.push((series_id, series_name, last_ts));
    }
    
    if updates.is_empty() {
        println!("ℹ️  Aucune série à compléter");
        return Task::none();
    }
    
    // Cloner le provider pour l'utiliser dans la Task async
    let provider = app.binance_provider.clone();
    
    // Calculer le timestamp actuel une seule fois
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| {
            eprintln!("⚠️ Erreur: horloge système invalide, utilisation d'un timestamp par défaut");
            std::time::Duration::from_secs(0)
        })
        .as_secs() as i64;
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage des requêtes async pour {} série(s)", updates.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = updates
                .into_iter()
                .map(|(series_id, series_name, last_ts)| {
                    let provider = provider.clone();
                    let series_id_clone = series_id.clone();
                    let series_name_clone = series_name.clone();
                    
                    async move {
                        let result = if let Some(last_timestamp) = last_ts {
                            // Extraire l'intervalle depuis le nom de la série (format: SYMBOL_INTERVAL)
                            let interval = series_name_clone.split('_').last().unwrap_or("1h");
                            
                            // Calculer le seuil pour déterminer si les données sont récentes (2 intervalles)
                            let threshold_seconds = calculate_candles_back_timestamp(interval, 2);
                            
                            // Si les données sont récentes (moins de 2 intervalles), on complète
                            // Sinon, on récupère depuis le dernier timestamp
                            let since_ts = if now - last_timestamp < threshold_seconds {
                                last_timestamp
                            } else {
                                // Si les données sont anciennes, on récupère les 100 dernières bougies
                                println!("  ℹ️  {}: Données anciennes, récupération des 100 dernières bougies", series_name_clone);
                                // Calculer dynamiquement selon l'intervalle
                                now - calculate_candles_back_timestamp(interval, 100)
                            };
                            
                            println!("  📥 {}: Récupération depuis le timestamp {}", series_name_clone, since_ts);
                            provider.fetch_new_candles_async(&series_id_clone, since_ts)
                                .await
                                .map_err(|e| e.to_string())
                        } else {
                            // Aucune donnée, synchroniser complètement
                            println!("  📥 {}: Aucune donnée, synchronisation complète", series_name_clone);
                            provider.fetch_all_candles_async(&series_id_clone)
                                .await
                                .map_err(|e| e.to_string())
                        };
                        
                        (series_id, series_name_clone, result)
                    }
                })
                .collect();
            
            // Exécuter toutes les requêtes en parallèle
            let results = join_all(futures).await;
            println!("✅ Toutes les requêtes async terminées");
            results
        },
        Message::CompleteMissingDataComplete,
    )
}

/// Applique les résultats de la complétion des données manquantes
pub fn apply_complete_missing_data_results(app: &mut ChartApp, results: Vec<(SeriesId, String, Result<Vec<Candle>, String>)>) -> Task<Message> {
    let mut has_updates = false;
    
    for (series_id, series_name, result) in results {
        match result {
            Ok(candles) => {
                if candles.is_empty() {
                    println!("  ℹ️  {}: Aucune nouvelle bougie", series_name);
                } else {
                    match app.chart_state.merge_candles(&series_id, candles) {
                        UpdateResult::MultipleCandlesAdded(n) => {
                            println!("  ✅ {}: {} nouvelles bougies ajoutées", series_name, n);
                            has_updates = true;
                        }
                        UpdateResult::Error(e) => {
                            println!("  ❌ {}: Erreur lors de la fusion - {}", series_name, e);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                println!("  ❌ {}: Erreur - {}", series_name, e);
            }
        }
    }
    
    // Après avoir complété les données manquantes, détecter et compléter les gaps internes
    if has_updates {
        println!("🔍 Vérification des gaps dans les données...");
        return complete_gaps(app);
    }
    
    println!("✅ Complétion terminée");
    Task::none()
}

/// Détecte et complète les gaps dans toutes les séries de manière asynchrone
pub fn complete_gaps(app: &mut ChartApp) -> Task<Message> {
    // Collecter toutes les informations nécessaires
    let mut gap_requests: Vec<(SeriesId, String, (i64, i64))> = Vec::new();
    
    for series in app.chart_state.series_manager.all_series() {
        let series_id = series.id.clone();
        let series_name = series.full_name();
        
        // Vérifier si le format est compatible avec Binance (SYMBOL_INTERVAL)
        if !is_binance_format(&series_name) {
            continue;
        }
        
        // Extraire l'intervalle depuis le nom de la série
        let interval_str = series_name.split('_').last().unwrap_or("1h");
        let interval_seconds = interval_to_seconds(interval_str);
        
        // Détecter les gaps
        let gaps = series.data.detect_gaps(interval_seconds);
        
        if !gaps.is_empty() {
            println!("  🔍 {}: {} gap(s) détecté(s)", series_name, gaps.len());
            // Ajouter chaque gap comme une requête séparée
            for gap in gaps {
                gap_requests.push((series_id.clone(), series_name.clone(), gap));
            }
        }
    }
    
    if gap_requests.is_empty() {
        println!("  ✅ Aucun gap détecté");
        return Task::none();
    }
    
    // Cloner le provider pour l'utiliser dans la Task async
    let provider = app.binance_provider.clone();
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage de la complétion des gaps pour {} gap(s)", gap_requests.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = gap_requests
                .into_iter()
                .map(|(series_id, series_name, (gap_start, gap_end))| {
                    let provider = provider.clone();
                    let series_id_clone = series_id.clone();
                    let series_name_clone = series_name.clone();
                    
                    async move {
                        println!("  📥 {}: Complétion du gap de {} à {}", series_name_clone, gap_start, gap_end);
                        let result = provider.fetch_candles_in_range_async(&series_id_clone, gap_start, gap_end)
                            .await
                            .map_err(|e| e.to_string());
                        (series_id, series_name_clone, (gap_start, gap_end), result)
                    }
                })
                .collect();
            
            // Exécuter toutes les requêtes en parallèle
            let results = join_all(futures).await;
            println!("✅ Toutes les requêtes de complétion des gaps terminées");
            results
        },
        Message::CompleteGapsComplete,
    )
}

/// Applique les résultats de la complétion des gaps
pub fn apply_complete_gaps_results(app: &mut ChartApp, results: Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>) -> Task<Message> {
    let mut has_updates = false;
    let mut updated_series: HashSet<SeriesId> = HashSet::new();
    
    for (series_id, series_name, (gap_start, gap_end), result) in results {
        match result {
            Ok(candles) => {
                if candles.is_empty() {
                    println!("    ℹ️  {}: Aucune bougie trouvée pour le gap de {} à {}", series_name, gap_start, gap_end);
                } else {
                    match app.chart_state.merge_candles(&series_id, candles) {
                        UpdateResult::MultipleCandlesAdded(n) => {
                            println!("    ✅ {}: {} bougies ajoutées pour combler le gap de {} à {}", series_name, n, gap_start, gap_end);
                            has_updates = true;
                            updated_series.insert(series_id);
                        }
                        UpdateResult::Error(e) => {
                            println!("    ❌ {}: Erreur lors de la fusion - {}", series_name, e);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                println!("    ❌ {}: Erreur lors de la récupération du gap de {} à {} - {}", series_name, gap_start, gap_end, e);
            }
        }
    }
    
    // Lancer la sauvegarde de manière asynchrone pour ne pas bloquer l'UI
    if !updated_series.is_empty() {
        return save_series_async(app, updated_series);
    }
    
    // Ajuster le viewport une seule fois à la fin (si auto-scroll activé)
    if has_updates && app.chart_style.auto_scroll_enabled {
        app.chart_state.auto_scroll_to_latest();
    }
    println!("✅ Complétion des gaps terminée");
    Task::none()
}

/// Sauvegarde les séries de manière asynchrone
fn save_series_async(app: &mut ChartApp, updated_series: HashSet<SeriesId>) -> Task<Message> {
    println!("💾 Lancement de la sauvegarde asynchrone des séries mises à jour...");
    
    // Collecter les données à sauvegarder (cloner ce qui est nécessaire)
    let save_requests: Vec<(String, String, String, Vec<Candle>)> = updated_series
        .iter()
        .filter_map(|series_id| {
            app.chart_state.series_manager.get_series(series_id)
                .map(|series| {
                    let file_path = format!("data/{}.json", series_id.name);
                    let symbol = series.symbol.clone();
                    let interval = series.interval.clone();
                    // Cloner toutes les bougies
                    let candles: Vec<Candle> = series.data.all_candles().to_vec();
                    (file_path, symbol, interval, candles)
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
            
            let futures: Vec<_> = save_requests
                .into_iter()
                .map(|(file_path, symbol, interval, candles)| {
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

/// Met à jour les données en temps réel pour les séries actives
pub fn update_realtime(app: &mut ChartApp) -> Task<Message> {
    if !app.realtime_enabled {
        return Task::none();
    }
    
    // Collecter les IDs des séries actives d'abord
    let active_series: Vec<(SeriesId, String)> = app.chart_state.series_manager
        .active_series()
        .filter_map(|s| {
            let name = s.full_name();
            // Vérifier si le format est compatible avec Binance
            if is_binance_format(&name) {
                Some((s.id.clone(), name))
            } else {
                None
            }
        })
        .collect();
    
    if active_series.is_empty() {
        return Task::none();
    }
    
    // Cloner le provider pour l'utiliser dans la Task async
    let provider = app.binance_provider.clone();
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage des requêtes async pour {} série(s)", active_series.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = active_series
                .iter()
                .map(|(series_id, series_name)| {
                    let provider = provider.clone();
                    let series_id = series_id.clone();
                    let series_name = series_name.clone();
                    
                    async move {
                        let result = provider.get_latest_candle_async(&series_id)
                            .await
                            .map_err(|e| e.to_string());
                        (series_id, series_name, result)
                    }
                })
                .collect();
            
            // Exécuter toutes les requêtes en parallèle
            let results = join_all(futures).await;
            println!("✅ Toutes les requêtes async terminées");
            results
        },
        Message::RealtimeUpdateComplete,
    )
}

/// Applique les résultats des mises à jour en temps réel
pub fn apply_realtime_updates(app: &mut ChartApp, results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>) {
    let mut has_updates = false;
    let mut has_new_candles = false;
    
    for (series_id, series_name, result) in results {
        match result {
            Ok(Some(candle)) => {
                match app.chart_state.update_candle(&series_id, candle) {
                    UpdateResult::NewCandle => {
                        println!("🔄 {}: Nouvelle bougie ajoutée", series_name);
                        has_updates = true;
                        has_new_candles = true;
                    }
                    UpdateResult::CandleUpdated => {
                        // Bougie mise à jour - on marque aussi comme update pour le re-render
                        has_updates = true;
                    }
                    UpdateResult::Error(e) => {
                        eprintln!("❌ {}: Erreur mise à jour - {}", series_name, e);
                    }
                    _ => {}
                }
            }
            Ok(None) => {
                // Aucune nouvelle bougie
            }
            Err(e) => {
                eprintln!("❌ {}: Erreur récupération - {}", series_name, e);
            }
        }
    }
    
    // Ajuster le viewport si nécessaire (si auto-scroll activé et nouvelles bougies)
    if has_new_candles && app.chart_style.auto_scroll_enabled {
        app.chart_state.auto_scroll_to_latest();
    }
    
    // Forcer le re-render en incrémentant le compteur de version
    // Note: Cette variable pourrait être utilisée dans le rendu du canvas pour forcer
    // un re-render explicite si nécessaire. Actuellement, Iced détecte automatiquement
    // les changements d'état, mais cette variable reste disponible pour un usage futur.
    if has_updates {
        app.render_version = app.render_version.wrapping_add(1);
    }
}

