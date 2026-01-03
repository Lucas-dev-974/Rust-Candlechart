//! Détection et complétion des gaps
//!
//! Ce module gère la détection des gaps dans les données
//! et leur complétion asynchrone depuis le provider.

use iced::Task;
use std::sync::Arc;
use crate::finance_chart::{
    UpdateResult,
    core::{SeriesId, Candle},
};
use crate::app::{
    messages::Message,
    utils::utils::interval_to_seconds,
    app_state::ChartApp,
    realtime::{
        realtime_utils::{is_binance_format, extract_interval, compute_fetch_since, calculate_recent_gap_threshold, current_timestamp},
        save::save_series_async,
    },
};

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
    
    // Arc::clone est très efficace (juste un compteur atomique)
    let provider = Arc::clone(&app.binance_provider);
    
    // Calculer le timestamp actuel une seule fois
    let now = current_timestamp();
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage des requêtes async pour {} série(s)", updates.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = updates
                .into_iter()
                .map(|(series_id, series_name, last_ts)| {
                    let provider = Arc::clone(&provider);
                    let series_id_clone = series_id.clone();
                    let series_name_clone = series_name.clone();
                    
                    async move {
                        let result = if let Some(last_timestamp) = last_ts {
                            // Extraire l'intervalle depuis le nom de la série (format: SYMBOL_INTERVAL)
                            let interval = extract_interval(&series_name_clone);
                            
                            // Utiliser la fonction pure pour déterminer depuis quand récupérer
                            let (since_ts, is_stale) = compute_fetch_since(last_timestamp, now, interval);
                            
                            if is_stale {
                                println!("  ℹ️  {}: Données anciennes, récupération des 100 dernières bougies", series_name_clone);
                            }
                            
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
    // Si aucune mise à jour, on peut calculer et stocker le MACD pour la série active
    let _ = app.chart_state.compute_and_store_macd();

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
        let interval_str = extract_interval(&series_name);
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
    
    // Arc::clone est très efficace (juste un compteur atomique)
    let provider = Arc::clone(&app.binance_provider);
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage de la complétion des gaps pour {} gap(s)", gap_requests.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = gap_requests
                .into_iter()
                .map(|(series_id, series_name, (gap_start, gap_end))| {
                    let provider = Arc::clone(&provider);
                    let series_id_clone = series_id.clone();
                    let series_name_clone = series_name.clone();
                    
                    async move {
                        let gap_days = (gap_end - gap_start) / 86400;
                        println!("  📥 {}: Complétion du gap de {} jours ({} à {})", series_name_clone, gap_days, gap_start, gap_end);
                        // Utiliser la version avec pagination pour les gros gaps
                        let result = provider.fetch_all_candles_in_range_async(&series_id_clone, gap_start, gap_end)
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
    let mut updated_series: std::collections::HashSet<SeriesId> = std::collections::HashSet::new();
    
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
        // Après les merges, recalculer et stocker le MACD avant d'éventuellement sauvegarder
        let _ = app.chart_state.compute_and_store_macd();
        return save_series_async(app, updated_series);
    }
    
    // Ajuster le viewport une seule fois à la fin (si auto-scroll activé)
    if has_updates && app.chart_style.auto_scroll_enabled {
        app.chart_state.auto_scroll_to_latest();
    }
    // Si des mises à jour ont eu lieu, stocker le cache MACD pour réutilisation
    if has_updates {
        let _ = app.chart_state.compute_and_store_macd();
    }
    println!("✅ Complétion des gaps terminée");
    Task::none()
}

/// Vérifie rapidement si une série a des gaps à combler (sans appel API)
/// Vérifie les gaps récents, internes, et si la série est vide
/// Note: Le gap historique (première bougie manquante) nécessite un appel API
/// et est vérifié dans auto_complete_series
pub fn has_gaps_to_fill(app: &ChartApp, series_id: &SeriesId) -> bool {
    if let Some(series) = app.chart_state.series_manager.get_series(series_id) {
        let name = series.full_name();
        if !is_binance_format(&name) {
            return false;
        }
        
        // Si la série est vide, il y a potentiellement un gap historique
        if series.data.len() == 0 {
            return true;
        }
        
        // Extraire l'intervalle pour calculer le seuil de gap récent
        let interval_str = extract_interval(&name);
        let interval_seconds = interval_to_seconds(interval_str);
        let threshold_seconds = calculate_recent_gap_threshold(interval_seconds);
        
        let newest = series.data.max_timestamp().unwrap_or(0);
        let now = current_timestamp();
        if newest > 0 && newest < now - threshold_seconds {
            return true;
        }
        
        // Vérifier les gaps internes
        let internal_gaps = series.data.detect_gaps(interval_seconds);
        if !internal_gaps.is_empty() {
            return true;
        }
    }
    false
}

/// Complète automatiquement une série avec toutes les données manquantes
/// Télécharge par batch de 1000 et met à jour le graphique progressivement
pub fn auto_complete_series(app: &mut ChartApp, series_id: SeriesId) -> Task<Message> {
    
    // Vérifier si le format est compatible avec Binance et extraire toutes les infos nécessaires
    let (series_name, current_oldest, current_newest, interval_seconds, interval_str, internal_gaps) = 
        if let Some(series) = app.chart_state.series_manager.get_series(&series_id) {
            let name = series.full_name();
            if !is_binance_format(&name) {
                println!("  ⚠️  {}: Format incompatible avec Binance", name);
                return Task::none();
            }
            
            // Récupérer les timestamps
            let oldest = series.data.min_timestamp().unwrap_or(0);
            let newest = series.data.max_timestamp().unwrap_or(0);
            
            // Extraire l'intervalle une seule fois (cloner le nom d'abord pour éviter les problèmes de borrow)
            let interval_str_value = extract_interval(&name).to_string();
            let interval_secs = interval_to_seconds(&interval_str_value);
            
            // Détecter les gaps internes maintenant (synchronement) car c'est rapide
            let gaps = series.data.detect_gaps(interval_secs);
            if !gaps.is_empty() {
                println!("  📊 {} gap(s) interne(s) détecté(s)", gaps.len());
            }
            
            (name, oldest, newest, interval_secs, interval_str_value, gaps)
        } else {
            eprintln!("❌ Série {} introuvable", series_id.name);
            return Task::none();
        };
    
    println!("🔄 Auto-complétion pour {}...", series_name);
    
    let provider = Arc::clone(&app.binance_provider);
    let series_id_clone = series_id.clone();
    let internal_gaps_clone = internal_gaps.clone();
    
    // Étape 1: Vérifier le timestamp le plus ancien disponible sur l'API et construire la liste des gaps
    Task::perform(
        async move {
            let api_oldest = match provider.check_oldest_available_timestamp_async(&series_id_clone).await {
                Ok(Some(ts)) => {
                    println!("  📅 Données disponibles depuis: {}", ts);
                    ts
                }
                Ok(None) => {
                    println!("  ⚠️ Impossible de déterminer les données historiques disponibles");
                    current_oldest
                }
                Err(e) => {
                    eprintln!("  ❌ Erreur API: {}", e);
                    current_oldest
                }
            };
            
            // Construire la liste de tous les gaps à combler
            // ORDRE: du plus récent vers le plus ancien (pour téléchargement progressif)
            let mut all_gaps = Vec::new();
            let now = current_timestamp();
            
            // 1. Gap récent (données jusqu'à maintenant) - PRIORITÉ ABSOLUE
            let threshold_seconds = calculate_recent_gap_threshold(interval_seconds);
            if current_newest > 0 && current_newest < now - threshold_seconds {
                let gap_minutes = (now - current_newest) / 60;
                let gap_hours = gap_minutes / 60;
                if gap_hours > 0 {
                    println!("  📥 Gap récent: {} heures ({} minutes)", gap_hours, gap_minutes);
                } else {
                    println!("  📥 Gap récent: {} minutes", gap_minutes);
                }
                all_gaps.push((current_newest, now));
            }
            
            // 2. Gaps internes - triés du PLUS RÉCENT au PLUS ANCIEN
            if !internal_gaps_clone.is_empty() {
                // Utiliser un Vec temporaire pour le tri (plus efficace que de cloner puis trier)
                let mut sorted_gaps = internal_gaps_clone;
                sorted_gaps.sort_unstable_by(|a, b| b.0.cmp(&a.0)); // Plus récent d'abord (sort_unstable est plus rapide)
                for (gap_start, gap_end) in sorted_gaps {
                    let gap_days = (gap_end - gap_start) / 86400;
                    println!("  📥 Gap interne: {} jours ({} -> {})", gap_days, gap_start, gap_end);
                    all_gaps.push((gap_start, gap_end));
                }
            }
            
            // 3. Gap historique - EN DERNIER (données les plus anciennes)
            if current_oldest == 0 {
                // Série vide : télécharger depuis le début jusqu'à maintenant
                let gap_days = (now - api_oldest) / 86400;
                println!("  📥 Gap historique: série vide, téléchargement depuis le début ({} jours)", gap_days);
                all_gaps.push((api_oldest, now));
            } else if api_oldest < current_oldest {
                // Il y a des données plus anciennes disponibles
                let gap_days = (current_oldest - api_oldest) / 86400;
                println!("  📥 Gap historique: {} jours (sera téléchargé en dernier)", gap_days);
                all_gaps.push((api_oldest, current_oldest));
            }
            
            if all_gaps.is_empty() {
                println!("  ✅ Série déjà complète!");
                return (series_id_clone, all_gaps, 0usize);
            }
            
            println!("  📊 {} plage(s) à télécharger", all_gaps.len());
            
            // Estimation du nombre total de bougies (utiliser l'interval_str déjà calculé)
            let estimated: usize = all_gaps.iter()
                .map(|(s, e)| crate::app::utils::utils::calculate_expected_candles(&interval_str, e - s))
                .sum();
            (series_id_clone, all_gaps, estimated)
        },
        |(series_id, gaps, estimated)| {
            if gaps.is_empty() {
                Message::DownloadComplete(series_id)
            } else {
                // Initialiser le téléchargement avec la liste des gaps
                Message::StartBatchDownload(series_id, gaps, estimated)
            }
        },
    )
}






