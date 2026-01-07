//! Handlers pour la gestion des actifs sélectionnés

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::persistence::SelectedAssetsPersistenceState;

/// Gère la sélection/désélection d'un actif
pub fn handle_toggle_asset_selection(
    app: &mut ChartApp,
    symbol: String,
) -> Task<crate::app::messages::Message> {
    if app.selected_assets.contains(&symbol) {
        app.selected_assets.remove(&symbol);
        println!("❌ Actif désélectionné: {}", symbol);
    } else {
        app.selected_assets.insert(symbol.clone());
        println!("✅ Actif sélectionné: {}", symbol);
    }
    
    // Sauvegarder les actifs sélectionnés
    let persistence_state = SelectedAssetsPersistenceState::from_hashset(&app.selected_assets);
    if let Err(e) = persistence_state.save_to_file("selected_assets.json") {
        eprintln!("⚠️ Erreur lors de la sauvegarde des actifs sélectionnés: {}", e);
    }
    
    Task::none()
}

/// Gère la sélection d'un actif depuis le header (change la série active)
pub fn handle_select_asset_from_header(
    app: &mut ChartApp,
    symbol: String,
) -> Task<crate::app::messages::Message> {
    use crate::app::handlers::series::handle_select_series_by_name;
    use crate::app::data::data_loading::download_series_for_symbol_and_interval;
    use std::sync::Arc;
    
    // Trouver une série correspondant à ce symbole
    // On cherche d'abord avec l'intervalle actif, sinon on prend la première trouvée
    let active_interval = app.chart_state.series_manager
        .active_series()
        .next()
        .map(|s| s.interval.clone())
        .unwrap_or_else(|| String::from("1h")); // Par défaut, utiliser 1h
    
    let series_name = app.chart_state.series_manager
        .all_series()
        .find(|series| {
            series.symbol == symbol
                && series.interval == active_interval
        })
        .or_else(|| {
            app.chart_state.series_manager
                .all_series()
                .find(|series| series.symbol == symbol)
        })
        .map(|series| series.full_name());
    
    if let Some(name) = series_name {
        println!("🔄 Changement de série vers: {}", name);
        handle_select_series_by_name(app, name)
    } else {
        // Aucune série trouvée, créer automatiquement la série avec l'intervalle actif
        println!("📥 Aucune série trouvée pour {}, création automatique avec l'intervalle {}...", symbol, active_interval);
        
        let provider = Arc::clone(&app.binance_provider);
        let symbol_for_task = symbol.clone();
        let interval_for_task = active_interval.clone();
        let symbol_for_message = symbol.clone();
        let interval_for_message = active_interval.clone();
        
        Task::perform(
            async move {
                download_series_for_symbol_and_interval(provider, &symbol_for_task, &interval_for_task).await
            },
            move |result| {
                crate::app::messages::Message::AssetSeriesCreated(symbol_for_message.clone(), interval_for_message.clone(), result)
            }
        )
    }
}

/// Gère le résultat de la création d'une série pour un actif
pub fn handle_asset_series_created(
    app: &mut ChartApp,
    symbol: String,
    interval: String,
    result: Result<crate::finance_chart::core::SeriesData, String>,
) -> Task<crate::app::messages::Message> {
    use crate::app::handlers::series::handle_select_series_by_name;
    
    match result {
        Ok(series) => {
            println!("✅ Série créée avec succès: {} ({} bougies)", series.full_name(), series.data.len());
            
            // Ajouter la série à l'application
            app.chart_state.add_series(series.clone());
            
            // Sélectionner automatiquement la nouvelle série
            handle_select_series_by_name(app, series.full_name())
        }
        Err(e) => {
            eprintln!("❌ Erreur lors de la création de la série {}_{}: {}", symbol, interval, e);
            Task::none()
        }
    }
}

