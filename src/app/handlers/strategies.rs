//! Handlers pour la gestion des stratégies de trading automatisées

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::strategies::strategy::{MarketContext, TradingSignal, TradingMode};
use crate::app::data::OrderType;

/// Sauvegarde automatiquement les stratégies
fn save_strategies(app: &ChartApp) {
    if let Err(e) = app.strategy_manager.save_to_file("strategies.json") {
        eprintln!("⚠️ Erreur lors de la sauvegarde des stratégies: {}", e);
    }
}

/// Exécute toutes les stratégies actives sur la série active
pub fn execute_strategies(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    // Récupérer la série active
    let Some(active_series) = app.chart_state.series_manager.active_series().next() else {
        return Task::none();
    };
    
    let symbol = active_series.symbol.clone();
    let series_id = active_series.id.clone();
    let interval = active_series.interval.clone();
    let candles = active_series.data.all_candles().to_vec();
    
    if candles.is_empty() {
        return Task::none();
    }
    
    let current_candle = candles.last().unwrap().clone();
    let current_price = current_candle.close;
    let current_volume = current_candle.volume;
    
    // Créer le contexte de marché
    let context = MarketContext {
        symbol: symbol.clone(),
        series_id,
        current_candle: current_candle.clone(),
        candles: candles.clone(),
        current_price,
        current_volume,
    };
    
    // Évaluer toutes les stratégies actives pour ce timeframe
    let results = app.strategy_manager.evaluate_all(&context, &interval);
    
    // Exécuter les signaux générés
    for (strategy_id, result) in results {
        // Récupérer la stratégie pour vérifier le mode de trading
        let Some(reg) = app.strategy_manager.get_strategy(&strategy_id) else {
            continue;
        };
        
        let strategy_name = reg.strategy.name().to_string();
        let trading_mode = reg.trading_mode;
        
        // Filtrer les signaux selon le mode de trading
        let signal = match (&result.signal, trading_mode) {
            (TradingSignal::Buy { .. }, TradingMode::SellOnly) => {
                // Ignorer les signaux d'achat si mode vente uniquement
                continue;
            }
            (TradingSignal::Sell { .. }, TradingMode::BuyOnly) => {
                // Ignorer les signaux de vente si mode achat uniquement
                continue;
            }
            _ => result.signal.clone(),
        };
        
        match signal {
            TradingSignal::Buy { quantity, order_type, take_profit, stop_loss, .. } => {
                println!("🤖 [{}] Signal d'achat: {} (confiance: {:.2}%)", 
                    strategy_id, result.reason, result.confidence * 100.0);
                
                if app.account_type.is_demo() {
                    let price = match order_type {
                        OrderType::Market => current_price,
                        OrderType::Limit => {
                            // Pour simplifier, on utilise le prix actuel
                            // Dans une vraie implémentation, on utiliserait limit_price
                            current_price
                        }
                    };
                    
                    let position = app.trading_state.trade_history.open_buy_position_with_tp_sl_and_strategy(
                        symbol.clone(),
                        quantity,
                        price,
                        take_profit,
                        stop_loss,
                        Some(strategy_id.clone()),
                        Some(strategy_name.clone()),
                    );
                    
                    println!("  ✅ Position ouverte automatiquement: Trade #{}", position.id);
                    
                    // Sauvegarder
                    if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                        eprintln!("⚠️ Erreur sauvegarde: {}", e);
                    }
                    
                    app.update_account_info();
                }
            }
            TradingSignal::Sell { quantity, order_type, take_profit, stop_loss, .. } => {
                println!("🤖 [{}] Signal de vente: {} (confiance: {:.2}%)", 
                    strategy_id, result.reason, result.confidence * 100.0);
                
                if app.account_type.is_demo() {
                    let price = match order_type {
                        OrderType::Market => current_price,
                        OrderType::Limit => current_price,
                    };
                    
                    // Essayer de fermer une position existante
                    if let Some(trade) = app.trading_state.trade_history.close_position_with_strategy(
                        &symbol, 
                        quantity, 
                        price,
                        Some(strategy_id.clone()),
                        Some(strategy_name.clone()),
                    ) {
                        println!("  ✅ Position fermée automatiquement: Trade #{} (P&L: {:.2})", 
                            trade.id, trade.realized_pnl);
                    } else {
                        // Ouvrir une position short
                        let trade = app.trading_state.trade_history.open_sell_position_with_tp_sl_and_strategy(
                            symbol.clone(),
                            quantity,
                            price,
                            take_profit,
                            stop_loss,
                            Some(strategy_id.clone()),
                            Some(strategy_name.clone()),
                        );
                        println!("  ✅ Position short ouverte automatiquement: Trade #{}", trade.id);
                    }
                    
                    if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                        eprintln!("⚠️ Erreur sauvegarde: {}", e);
                    }
                    
                    app.update_account_info();
                }
            }
            TradingSignal::Hold => {
                // Ne rien faire - on peut logger si nécessaire
            }
        }
    }
    
    Task::none()
}

/// Enregistre une stratégie RSI
pub fn handle_register_rsi_strategy(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::strategies::examples::RSIStrategy;
    let strategy = Box::new(RSIStrategy::new());
    let id = app.strategy_manager.register_strategy(strategy);
    println!("✅ Stratégie RSI enregistrée: {}", id);
    save_strategies(app);
    Task::none()
}

/// Enregistre une stratégie MA Crossover
pub fn handle_register_ma_crossover_strategy(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::strategies::examples::MovingAverageCrossoverStrategy;
    let strategy = Box::new(MovingAverageCrossoverStrategy::new());
    let id = app.strategy_manager.register_strategy(strategy);
    println!("✅ Stratégie MA Crossover enregistrée: {}", id);
    save_strategies(app);
    Task::none()
}

/// Active une stratégie
pub fn handle_enable_strategy(app: &mut ChartApp, id: String) -> Task<crate::app::messages::Message> {
    if let Err(e) = app.strategy_manager.enable_strategy(&id) {
        eprintln!("❌ Erreur activation stratégie: {}", e);
    } else {
        println!("✅ Stratégie {} activée", id);
        save_strategies(app);
    }
    Task::none()
}

/// Désactive une stratégie
pub fn handle_disable_strategy(app: &mut ChartApp, id: String) -> Task<crate::app::messages::Message> {
    if let Err(e) = app.strategy_manager.disable_strategy(&id) {
        eprintln!("❌ Erreur désactivation stratégie: {}", e);
    } else {
        println!("✅ Stratégie {} désactivée", id);
        save_strategies(app);
    }
    Task::none()
}

/// Supprime une stratégie
pub fn handle_remove_strategy(app: &mut ChartApp, id: String) -> Task<crate::app::messages::Message> {
    if let Err(e) = app.strategy_manager.remove_strategy(&id) {
        eprintln!("❌ Erreur suppression stratégie: {}", e);
    } else {
        println!("✅ Stratégie {} supprimée", id);
        save_strategies(app);
    }
    Task::none()
}

/// Ouvre ou ferme le panneau de configuration d'une stratégie
pub fn handle_toggle_strategy_config(app: &mut ChartApp, strategy_id: String) -> Task<crate::app::messages::Message> {
    use crate::app::app_state::StrategyEditingState;
    use std::collections::HashMap;
    
    let strategy_id_clone = strategy_id.clone();
    let editing_state = app.editing_strategies.entry(strategy_id).or_insert_with(|| {
        // Initialiser l'état d'édition avec les valeurs actuelles
        let reg = app.strategy_manager.get_strategy(&strategy_id_clone);
        let mut param_values = HashMap::new();
        let selected_timeframes = reg
            .and_then(|r| r.allowed_timeframes.clone())
            .unwrap_or_default();
        
        if let Some(reg) = reg {
            for param in reg.strategy.parameters() {
                param_values.insert(param.name, format!("{:.2}", param.value));
            }
        }
        
        StrategyEditingState {
            expanded: false,
            param_values,
            selected_timeframes,
            trading_mode: reg.map(|r| r.trading_mode).unwrap_or(crate::app::strategies::strategy::TradingMode::Both),
        }
    });
    
    editing_state.expanded = !editing_state.expanded;
    
    Task::none()
}

/// Met à jour la valeur temporaire d'un paramètre dans l'input
pub fn handle_update_strategy_param_input(
    app: &mut ChartApp,
    strategy_id: String,
    param_name: String,
    value: String,
) -> Task<crate::app::messages::Message> {
    use crate::app::app_state::StrategyEditingState;
    use std::collections::HashMap;
    
    let strategy_id_clone = strategy_id.clone();
    let reg = app.strategy_manager.get_strategy(&strategy_id_clone);
    let editing_state = app.editing_strategies.entry(strategy_id).or_insert_with(|| {
        StrategyEditingState {
            expanded: true,
            param_values: HashMap::new(),
            selected_timeframes: Vec::new(),
            trading_mode: reg.map(|r| r.trading_mode).unwrap_or(crate::app::strategies::strategy::TradingMode::Both),
        }
    });
    
    editing_state.param_values.insert(param_name, value);
    
    Task::none()
}

/// Ajoute ou retire un timeframe de la sélection temporaire
pub fn handle_toggle_strategy_timeframe(
    app: &mut ChartApp,
    strategy_id: String,
    timeframe: String,
) -> Task<crate::app::messages::Message> {
    use crate::app::app_state::StrategyEditingState;
    use std::collections::HashMap;
    
    let strategy_id_clone = strategy_id.clone();
    let editing_state = app.editing_strategies.entry(strategy_id).or_insert_with(|| {
        // Initialiser avec les timeframes actuels
        let reg = app.strategy_manager.get_strategy(&strategy_id_clone);
        let selected_timeframes = reg
            .and_then(|r| r.allowed_timeframes.clone())
            .unwrap_or_default();
        
        StrategyEditingState {
            expanded: true,
            param_values: HashMap::new(),
            selected_timeframes,
            trading_mode: reg.map(|r| r.trading_mode).unwrap_or(crate::app::strategies::strategy::TradingMode::Both),
        }
    });
    
    // Ajouter ou retirer le timeframe
    if let Some(pos) = editing_state.selected_timeframes.iter().position(|tf| tf == &timeframe) {
        editing_state.selected_timeframes.remove(pos);
    } else {
        editing_state.selected_timeframes.push(timeframe);
    }
    
    Task::none()
}

/// Applique les modifications d'une stratégie
pub fn handle_apply_strategy_config(app: &mut ChartApp, strategy_id: String) -> Task<crate::app::messages::Message> {
    // Récupérer l'état d'édition
    let Some(editing_state) = app.editing_strategies.get(&strategy_id) else {
        return Task::none();
    };
    
    // Récupérer la stratégie
    let Some(reg) = app.strategy_manager.get_strategy_mut(&strategy_id) else {
        eprintln!("❌ Stratégie {} introuvable", strategy_id);
        return Task::none();
    };
    
    // Appliquer les paramètres
    for param in reg.strategy.parameters() {
        if let Some(value_str) = editing_state.param_values.get(&param.name) {
            if let Ok(value) = value_str.parse::<f64>() {
                if value >= param.min && value <= param.max {
                    if let Err(e) = reg.strategy.update_parameter(&param.name, value) {
                        eprintln!("⚠️ Erreur mise à jour paramètre {}: {}", param.name, e);
                    } else {
                        println!("✅ Paramètre {} mis à jour: {:.2}", param.name, value);
                    }
                } else {
                    eprintln!("⚠️ Valeur {} hors limites pour {} (min: {}, max: {})", 
                        value, param.name, param.min, param.max);
                }
            } else {
                eprintln!("⚠️ Valeur invalide pour {}: {}", param.name, value_str);
            }
        }
    }
    
    // Appliquer les timeframes
    let timeframes = if editing_state.selected_timeframes.is_empty() {
        None
    } else {
        Some(editing_state.selected_timeframes.clone())
    };
    
    if let Err(e) = app.strategy_manager.set_strategy_timeframes(&strategy_id, timeframes.clone()) {
        eprintln!("❌ Erreur mise à jour timeframes: {}", e);
    } else {
        match &timeframes {
            Some(tfs) => {
                println!("✅ Timeframes mis à jour: {:?}", tfs);
            }
            None => {
                println!("✅ Tous les timeframes autorisés");
            }
        }
    }
    
    // Appliquer le mode de trading
    if let Err(e) = app.strategy_manager.set_strategy_trading_mode(&strategy_id, editing_state.trading_mode) {
        eprintln!("❌ Erreur mise à jour mode de trading: {}", e);
    } else {
        let mode_text = match editing_state.trading_mode {
            crate::app::strategies::strategy::TradingMode::BuyOnly => "Achats uniquement",
            crate::app::strategies::strategy::TradingMode::SellOnly => "Ventes uniquement",
            crate::app::strategies::strategy::TradingMode::Both => "Achats et ventes",
        };
        println!("✅ Mode de trading mis à jour: {}", mode_text);
    }
    
    // Fermer le panneau de configuration
    if let Some(editing) = app.editing_strategies.get_mut(&strategy_id) {
        editing.expanded = false;
    }
    
    // Sauvegarder les modifications
    save_strategies(app);
    
    Task::none()
}

/// Annule les modifications d'une stratégie
pub fn handle_cancel_strategy_config(app: &mut ChartApp, strategy_id: String) -> Task<crate::app::messages::Message> {
    // Fermer le panneau de configuration et réinitialiser l'état
    if let Some(editing) = app.editing_strategies.get_mut(&strategy_id) {
        editing.expanded = false;
        // Réinitialiser les valeurs avec les valeurs actuelles
        if let Some(reg) = app.strategy_manager.get_strategy(&strategy_id) {
            editing.param_values.clear();
            for param in reg.strategy.parameters() {
                editing.param_values.insert(param.name, format!("{:.2}", param.value));
            }
            editing.selected_timeframes = reg.allowed_timeframes.clone().unwrap_or_default();
            editing.trading_mode = reg.trading_mode;
        }
    }
    
    Task::none()
}

/// Met à jour le mode de trading temporairement dans l'état d'édition
pub fn handle_update_strategy_trading_mode(
    app: &mut ChartApp,
    strategy_id: String,
    trading_mode: crate::app::strategies::strategy::TradingMode,
) -> Task<crate::app::messages::Message> {
    use crate::app::app_state::StrategyEditingState;
    use std::collections::HashMap;
    
    let editing_state = app.editing_strategies.entry(strategy_id).or_insert_with(|| {
        StrategyEditingState {
            expanded: true,
            param_values: HashMap::new(),
            selected_timeframes: Vec::new(),
            trading_mode: crate::app::strategies::strategy::TradingMode::Both,
        }
    });
    
    editing_state.trading_mode = trading_mode;
    
    Task::none()
}


