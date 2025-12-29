//! Gestion du temps réel et de la complétion des données
//!
//! Ce module gère toutes les opérations asynchrones liées à la mise à jour
//! en temps réel des données et à la complétion des gaps.
//!
//! La logique pure (fonctions sans effets de bord) est extraite dans
//! le module `realtime_utils` pour faciliter les tests.

use iced::Task;
use std::collections::HashSet;
use std::sync::Arc;
use crate::finance_chart::{
    UpdateResult,
    core::{SeriesId, Candle},
};
use crate::app::{
    messages::Message,
    utils::{interval_to_seconds, calculate_expected_candles},
    app_state::ChartApp,
    realtime_utils::{is_binance_format, extract_interval, compute_fetch_since, calculate_recent_gap_threshold, current_timestamp},
};

/// Charge l'historique complet d'une série depuis Binance
pub fn load_full_history(app: &mut ChartApp, series_id: SeriesId) -> Task<Message> {
    // Vérifier si le format est compatible avec Binance
    let series_name = if let Some(series) = app.chart_state.series_manager.get_series(&series_id) {
        let name = series.full_name();
        if !is_binance_format(&name) {
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

/// Sauvegarde les séries de manière asynchrone
pub fn save_series_async(app: &mut ChartApp, updated_series: HashSet<SeriesId>) -> Task<Message> {
    println!("💾 Lancement de la sauvegarde asynchrone des séries mises à jour...");
    
    // Collecter les données à sauvegarder (cloner ce qui est nécessaire)
    let save_requests: Vec<(String, String, String, Vec<Candle>, std::path::PathBuf)> = updated_series
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
                    let candles: Vec<Candle> = series.data.all_candles().to_vec();
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
    
    // Arc::clone est très efficace (juste un compteur atomique)
    let provider = Arc::clone(&app.binance_provider);
    
    // Créer une Task async qui fait toutes les requêtes en parallèle
    println!("🚀 Démarrage des requêtes async pour {} série(s)", active_series.len());
    Task::perform(
        async move {
            use futures::future::join_all;
            
            // Créer un vecteur de futures pour toutes les requêtes
            let futures: Vec<_> = active_series
                .iter()
                .map(|(series_id, series_name)| {
                    let provider = Arc::clone(&provider);
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
        // Mettre à jour le cache MACD centralisé après les mises à jour temps réel
        let _ = app.chart_state.compute_and_store_macd();
    }
}

/// Teste la connexion au provider actif
pub fn test_provider_connection(app: &ChartApp) -> Task<Message> {
    let provider = Arc::clone(&app.binance_provider);
    let has_token = app.provider_config
        .active_config()
        .map(|c| c.api_token.is_some())
        .unwrap_or(false);
    
    println!("🔍 Test de connexion au provider...");
    
    Task::perform(
        async move {
            // Si un token est configuré, tester l'authentification
            // Sinon, tester juste la connexion de base
            if has_token {
                provider.test_authenticated_connection().await
                    .map_err(|e| e.to_string())
            } else {
                provider.test_connection().await
                    .map_err(|e| e.to_string())
            }
        },
        Message::ProviderConnectionTestComplete,
    )
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
                .map(|(s, e)| calculate_expected_candles(&interval_str, e - s))
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

