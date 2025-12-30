//! Mises à jour en temps réel
//!
//! Ce module gère les mises à jour périodiques des données en temps réel
//! pour les séries actives.

use iced::Task;
use std::sync::Arc;
use crate::finance_chart::{
    UpdateResult,
    core::{SeriesId, Candle},
};
use crate::app::{
    messages::Message,
    app_state::ChartApp,
    realtime::realtime_utils::is_binance_format,
};

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
    
    // Collecter les symboles et prix avant de traiter les résultats
    let mut symbol_prices = Vec::new();
    
    for (series_id, series_name, result) in &results {
        match result {
            Ok(Some(candle)) => {
                // Collecter le symbole et le prix pour la vérification des ordres
                if let Some(series) = app.chart_state.series_manager.get_series(series_id) {
                    symbol_prices.push((series.symbol.clone(), candle.close));
                }
                
                match app.chart_state.update_candle(series_id, candle.clone()) {
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
        
        // Mettre à jour les informations du compte (P&L non réalisé) si on est en mode paper trading
        if app.account_type.is_demo() && has_updates {
            // Vérifier et exécuter les ordres limit en attente et les TP/SL
            for (symbol, current_price) in &symbol_prices {
                // Vérifier les ordres limit en attente
                app.trading_state.trade_history.check_and_execute_pending_orders(symbol, *current_price);
                
                // Vérifier les TP/SL des positions ouvertes
                app.trading_state.trade_history.check_take_profit_stop_loss(symbol, *current_price);
            }
            
            // Mettre à jour automatiquement TP/SL avec 15% d'écart si les champs sont vides
            if let Some(current_price) = symbol_prices.first().map(|(_, price)| *price) {
                app.trading_state.update_tp_sl_from_price(current_price);
            }
            
            app.update_account_info();
            
            // Sauvegarder l'historique si des ordres ont été exécutés ou des positions fermées
            if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                eprintln!("⚠️ Erreur sauvegarde historique trading: {}", e);
            }
        } else if app.account_type.is_demo() {
            app.update_account_info();
        }
    }
}

