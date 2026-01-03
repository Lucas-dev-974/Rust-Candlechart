//! Handlers pour la gestion des séries et chargement de données

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::persistence::TimeframePersistenceState;

/// Gère la sélection d'une série par nom
pub fn handle_select_series_by_name(app: &mut ChartApp, series_name: String) -> Task<crate::app::messages::Message> {
    
    println!("🔄 Sélection de la série: {}", series_name);
    
    // Trouver le SeriesId correspondant au nom
    let series_id_opt = app.chart_state.series_manager.all_series()
        .find(|s| s.full_name() == series_name)
        .map(|s| s.id.clone());
    
    if let Some(series_id) = series_id_opt {
        // Activer uniquement cette série (désactive toutes les autres)
        app.chart_state.series_manager.activate_only_series(series_id.clone());
        // Mettre à jour le viewport après activation
        app.chart_state.update_viewport_from_series();
        
        // Sauvegarder le timeframe sélectionné
        if let Some(series) = app.chart_state.series_manager.get_series(&series_id) {
            let timeframe_state = TimeframePersistenceState {
                interval: series.interval.clone(),
                symbol: Some(series.symbol.clone()),
            };
            if let Err(e) = timeframe_state.save_to_file("timeframe.json") {
                eprintln!("⚠️ Erreur sauvegarde timeframe: {}", e);
            }
        }
        
        // Mettre à jour automatiquement TP/SL avec 15% d'écart si les champs sont vides
        if let Some(current_price) = app.chart_state.series_manager
            .active_series()
            .next()
            .and_then(|s| s.data.last_candle().map(|c| c.close))
        {
            app.trading_state.update_tp_sl_from_price(current_price);
        }
        
        // Vérifier automatiquement les gaps de la série
        // et télécharger les données manquantes (historique + gaps)
        if let Some(series) = app.chart_state.series_manager.get_series(&series_id) {
            let current_count = series.data.len();
            let oldest = series.data.min_timestamp();
            
            println!("🔍 Vérification série {}: {} bougies", series_name, current_count);
            if let Some(ts) = oldest {
                println!("  📅 Première bougie: {}", ts);
            }
            
            // Vérifier s'il y a des gaps à combler (récent, internes, ou historique)
            // has_gaps_to_fill vérifie déjà si la série est vide
            let has_gaps = crate::app::realtime::has_gaps_to_fill(app, &series_id);
            
            if has_gaps {
                println!("📥 Série {} a des gaps à combler, lancement de l'auto-complétion...", series_name);
                return crate::app::realtime::auto_complete_series(app, series_id);
            } else {
                println!("✅ Série {} complète ({} bougies, pas de gaps)", series_name, current_count);
            }
        }
    }
    Task::none()
}

/// Gère le chargement des séries depuis le répertoire
pub fn handle_load_series_complete(
    app: &mut ChartApp,
    result: Result<Vec<crate::finance_chart::core::SeriesData>, String>
) -> Task<crate::app::messages::Message> {
    
    match result {
        Ok(series_list) => {
            for series in series_list {
                let series_name = series.full_name();
                println!(
                    "  📊 {}: {} bougies ({} - {})",
                    series_name,
                    series.data.len(),
                    series.symbol,
                    series.interval
                );
                app.chart_state.add_series(series);
            }
            // Calculer et stocker le MACD pré-calculé une fois après le chargement initial
            let _ = app.chart_state.compute_and_store_macd();
            if app.chart_state.series_manager.total_count() == 0 {
                eprintln!("⚠️ Aucune série chargée. Vérifiez que le dossier 'data' contient des fichiers JSON.");
                return Task::none();
            }
            
            // Restaurer le timeframe sauvegardé
            if let Some(saved_interval) = crate::app::state::loaders::load_timeframe() {
                // Chercher une série avec l'intervalle sauvegardé
                // Prioriser le symbole sauvegardé si disponible
                let saved_symbol = TimeframePersistenceState::load_from_file("timeframe.json")
                    .ok()
                    .and_then(|state| state.symbol);
                
                let series_to_activate = if let Some(ref symbol) = saved_symbol {
                    // Chercher d'abord avec le symbole sauvegardé
                    app.chart_state.series_manager.all_series()
                        .find(|s| s.interval == saved_interval && s.symbol == *symbol)
                        .or_else(|| {
                            // Sinon chercher avec n'importe quel symbole
                            app.chart_state.series_manager.all_series()
                                .find(|s| s.interval == saved_interval)
                        })
                } else {
                    // Chercher avec n'importe quel symbole
                    app.chart_state.series_manager.all_series()
                        .find(|s| s.interval == saved_interval)
                };
                
                if let Some(series) = series_to_activate {
                    println!("🔄 Restauration du timeframe sauvegardé: {}", series.full_name());
                    app.chart_state.series_manager.activate_only_series(series.id.clone());
                    app.chart_state.update_viewport_from_series();
                } else {
                    println!("⚠️ Timeframe sauvegardé '{}' non trouvé, utilisation de la série par défaut", saved_interval);
                }
            }
            
            // Initialiser TP/SL avec 15% d'écart du prix actuel si les champs sont vides
            if let Some(current_price) = app.chart_state.series_manager
                .active_series()
                .next()
                .and_then(|s| s.data.last_candle().map(|c| c.close))
            {
                app.trading_state.update_tp_sl_from_price(current_price);
            }
            
            // Vérifier si la série active a des gaps à combler
            let active_series_info = app.chart_state.series_manager.active_series()
                .next()
                .map(|s| {
                    let oldest = s.data.min_timestamp();
                    (s.id.clone(), s.full_name(), s.data.len(), oldest)
                });
            
            if let Some((series_id, series_name, candle_count, oldest)) = active_series_info {
                println!("🔍 Vérification série active {}: {} bougies", series_name, candle_count);
                if let Some(ts) = oldest {
                    println!("  📅 Première bougie: {}", ts);
                }
                
                // Vérifier s'il y a des gaps à combler (récent, internes, ou historique)
                // has_gaps_to_fill vérifie déjà si la série est vide
                let has_gaps = crate::app::realtime::has_gaps_to_fill(app, &series_id);
                
                if has_gaps {
                    println!("📥 Série active {} a des gaps à combler, lancement de l'auto-complétion...", series_name);
                    return crate::app::realtime::auto_complete_series(app, series_id);
                } else {
                    println!("✅ Série active {} complète ({} bougies, pas de gaps)", series_name, candle_count);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur lors du chargement des séries: {}", e);
        }
    }
    // Mettre à jour le compte après le chargement des séries (pour avoir le prix actuel)
    if app.account_type.is_demo() {
        app.update_account_info();
    }
    Task::none()
}

