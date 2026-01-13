//! Handlers pour la gestion du trading

use iced::Task;
use std::sync::Arc;
use crate::app::app_state::ChartApp;
use crate::app::data::OrderType;
use crate::app::trading::api::{validate_order, place_market_buy_order, place_market_sell_order, place_limit_buy_order, place_limit_sell_order};

/// Gère la mise à jour de la quantité d'ordre
pub fn handle_update_order_quantity(app: &mut ChartApp, quantity: String) -> Task<crate::app::messages::Message> {
    app.trading_state.order_quantity = quantity;
    Task::none()
}

/// Gère la mise à jour du type d'ordre
pub fn handle_update_order_type(app: &mut ChartApp, order_type: OrderType) -> Task<crate::app::messages::Message> {
    app.trading_state.order_type = order_type;
    // Si on passe en Market, réinitialiser le prix limite et désactiver TP/SL
    if order_type == OrderType::Market {
        app.trading_state.limit_price = String::new();
        app.trading_state.tp_sl_enabled = false;
    } else if app.trading_state.limit_price.is_empty() {
        // Si on passe en Limit et que le prix limite est vide, l'initialiser avec le prix actuel
        if let Some(price) = app.chart_state.series_manager
            .active_series()
            .next()
            .and_then(|s| s.data.last_candle().map(|c| c.close))
        {
            app.trading_state.limit_price = format!("{:.2}", price);
        }
    }
    Task::none()
}

/// Gère la mise à jour du prix limite
pub fn handle_update_limit_price(app: &mut ChartApp, price: String) -> Task<crate::app::messages::Message> {
    app.trading_state.limit_price = price;
    Task::none()
}

/// Gère la mise à jour du take profit
pub fn handle_update_take_profit(app: &mut ChartApp, tp: String) -> Task<crate::app::messages::Message> {
    app.trading_state.take_profit = tp;
    Task::none()
}

/// Gère la mise à jour du stop loss
pub fn handle_update_stop_loss(app: &mut ChartApp, sl: String) -> Task<crate::app::messages::Message> {
    app.trading_state.stop_loss = sl;
    Task::none()
}

/// Gère le toggle de TP/SL
pub fn handle_toggle_tp_sl_enabled(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.trading_state.tp_sl_enabled = !app.trading_state.tp_sl_enabled;
    Task::none()
}

/// Gère le placement d'un ordre d'achat
pub fn handle_place_buy_order(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    if let Some(quantity) = app.trading_state.parse_quantity() {
        if quantity > 0.0 {
            // Utiliser le symbole mémorisé depuis le pick_list si disponible, sinon le symbole de la série active
            let symbol = app.selected_asset_symbol
                .clone()
                .or_else(|| {
                    app.chart_state.series_manager
                        .active_series()
                        .next()
                        .map(|s| s.symbol.clone())
                })
                .unwrap_or_else(|| String::from("UNKNOWN"));
            
            // Récupérer le prix actuel de la série correspondant au symbole mémorisé
            let current_price = {
                let active_series = app.chart_state.series_manager.active_series().next();
                if let Some(ref selected_symbol) = app.selected_asset_symbol {
                    if let Some(active) = active_series {
                        if active.symbol == *selected_symbol {
                            active.data.last_candle().map(|c| c.close).unwrap_or(0.0)
                        } else {
                            app.chart_state.series_manager
                                .all_series()
                                .find(|s| s.symbol == *selected_symbol && s.interval == active.interval)
                                .or_else(|| {
                                    app.chart_state.series_manager
                                        .all_series()
                                        .find(|s| s.symbol == *selected_symbol)
                                })
                                .and_then(|s| s.data.last_candle().map(|c| c.close))
                                .unwrap_or(0.0)
                        }
                    } else {
                        app.chart_state.series_manager
                            .all_series()
                            .find(|s| s.symbol == *selected_symbol)
                            .and_then(|s| s.data.last_candle().map(|c| c.close))
                            .unwrap_or(0.0)
                    }
                } else {
                    active_series
                        .and_then(|s| s.data.last_candle().map(|c| c.close))
                        .unwrap_or(0.0)
                }
            };
            
            let (price, total_amount) = match app.trading_state.order_type {
                OrderType::Market => {
                    let total = quantity * current_price;
                    (current_price, total)
                }
                OrderType::Limit => {
                    if let Some(limit_price) = app.trading_state.parse_limit_price() {
                        if limit_price > 0.0 {
                            let total = quantity * limit_price;
                            (limit_price, total)
                        } else {
                            println!("❌ Prix limite invalide");
                            return Task::none();
                        }
                    } else {
                        println!("❌ Prix limite invalide");
                        return Task::none();
                    }
                }
            };
            
            // Récupérer TP et SL (en mode Market, vérifier la checkbox)
            let take_profit = if app.trading_state.order_type == OrderType::Market && !app.trading_state.tp_sl_enabled {
                None
            } else {
                app.trading_state.parse_take_profit()
            };
            let stop_loss = if app.trading_state.order_type == OrderType::Market && !app.trading_state.tp_sl_enabled {
                None
            } else {
                app.trading_state.parse_stop_loss()
            };
            
            // Vérifier si on a assez de marge libre
            if total_amount <= app.account_info.free_margin {
                match app.trading_state.order_type {
                    OrderType::Market => {
                        println!("📈 Ordre d'achat MARKET: {} {} à {:.2} USDT (Total: {:.2} USDT)", 
                            quantity, symbol, price, total_amount);
                        
                        // En mode démo, simuler l'ordre
                        if app.account_type.is_demo() {
                            // Récupérer le timestamp de la dernière bougie
                            let timestamp = app.chart_state.series_manager
                                .active_series()
                                .next()
                                .and_then(|s| s.data.last_candle().map(|c| c.timestamp));
                            
                            // Ouvrir une position d'achat avec TP/SL
                            let position = app.trading_state.trade_history.open_buy_position_with_tp_sl_and_strategy(
                                symbol.clone(),
                                quantity,
                                price,
                                take_profit,
                                stop_loss,
                                None,
                                None,
                                timestamp,
                            );
                            
                            println!("  ✅ Position ouverte: Trade #{}", position.id);
                            if take_profit.is_some() || stop_loss.is_some() {
                                println!("  📊 TP: {:?}, SL: {:?}", take_profit, stop_loss);
                            }
                            
                            // Sauvegarder l'historique
                            if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                                eprintln!("⚠️ Erreur sauvegarde historique trading: {}", e);
                            }
                            
                            // Mettre à jour les informations du compte
                            app.update_account_info();
                        } else {
                            // Mode réel: placer un vrai ordre MARKET BUY via l'API du provider
                            let provider = Arc::clone(&app.binance_provider);
                            let symbol_clone = symbol.clone();
                            let quantity_clone = quantity;
                            
                            // Valider l'ordre avant de le placer
                            if let Err(e) = validate_order(
                                &symbol_clone,
                                quantity_clone,
                                Some(price),
                                "MARKET",
                                app.account_info.free_margin,
                            ) {
                                println!("  ❌ Validation échouée: {}", e);
                                return Task::none();
                            }
                            
                            return Task::perform(
                                async move {
                                    place_market_buy_order(&provider, &symbol_clone, quantity_clone)
                                        .await
                                        .map_err(|e| e.to_string())
                                },
                                crate::app::messages::Message::BuyOrderPlaced,
                            );
                        }
                    }
                    OrderType::Limit => {
                        println!("📈 Ordre LIMIT d'achat: {} {} à {:.2} USDT (Total: {:.2} USDT)", 
                            quantity, symbol, price, total_amount);
                        
                        // En mode démo, créer un ordre en attente
                        if app.account_type.is_demo() {
                            let order = app.trading_state.trade_history.create_pending_order(
                                symbol.clone(),
                                crate::app::data::TradeType::Buy,
                                quantity,
                                price,
                                take_profit,
                                stop_loss,
                            );
                            println!("  ✅ Ordre limit créé: Order #{} (sera exécuté si prix <= {:.2})", 
                                order.id, price);
                            
                            // Vérifier immédiatement si l'ordre peut être exécuté
                            let timestamp = app.chart_state.series_manager
                                .active_series()
                                .next()
                                .and_then(|s| s.data.last_candle().map(|c| c.timestamp));
                            app.trading_state.trade_history.check_and_execute_pending_orders(&symbol, current_price, timestamp);
                            
                            // Sauvegarder l'historique
                            if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                                eprintln!("⚠️ Erreur sauvegarde historique trading: {}", e);
                            }
                            
                            // Mettre à jour les informations du compte
                            app.update_account_info();
                        } else {
                            // Mode réel: placer un vrai ordre LIMIT BUY via l'API du provider
                            let provider = Arc::clone(&app.binance_provider);
                            let symbol_clone = symbol.clone();
                            let quantity_clone = quantity;
                            let price_clone = price;
                            
                            // Valider l'ordre avant de le placer
                            if let Err(e) = validate_order(
                                &symbol_clone,
                                quantity_clone,
                                Some(price_clone),
                                "LIMIT",
                                app.account_info.free_margin,
                            ) {
                                println!("  ❌ Validation échouée: {}", e);
                                return Task::none();
                            }
                            
                            return Task::perform(
                                async move {
                                    place_limit_buy_order(&provider, &symbol_clone, quantity_clone, price_clone, Some("GTC"))
                                        .await
                                        .map_err(|e| e.to_string())
                                },
                                crate::app::messages::Message::BuyOrderPlaced,
                            );
                        }
                    }
                }
            } else {
                println!("❌ Ordre d'achat refusé: marge insuffisante (nécessaire: {:.2} USDT, disponible: {:.2} USDT)", 
                    total_amount, app.account_info.free_margin);
            }
        } else {
            println!("❌ Quantité invalide: {}", quantity);
        }
    } else {
        println!("❌ Quantité invalide: {}", app.trading_state.order_quantity);
    }
    Task::none()
}

/// Gère le placement d'un ordre de vente
pub fn handle_place_sell_order(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    if let Some(quantity) = app.trading_state.parse_quantity() {
        if quantity > 0.0 {
            // Utiliser le symbole mémorisé depuis le pick_list si disponible, sinon le symbole de la série active
            let symbol = app.selected_asset_symbol
                .clone()
                .or_else(|| {
                    app.chart_state.series_manager
                        .active_series()
                        .next()
                        .map(|s| s.symbol.clone())
                })
                .unwrap_or_else(|| String::from("UNKNOWN"));
            
            // Récupérer le prix actuel de la série correspondant au symbole mémorisé
            let current_price = {
                let active_series = app.chart_state.series_manager.active_series().next();
                if let Some(ref selected_symbol) = app.selected_asset_symbol {
                    if let Some(active) = active_series {
                        if active.symbol == *selected_symbol {
                            active.data.last_candle().map(|c| c.close).unwrap_or(0.0)
                        } else {
                            app.chart_state.series_manager
                                .all_series()
                                .find(|s| s.symbol == *selected_symbol && s.interval == active.interval)
                                .or_else(|| {
                                    app.chart_state.series_manager
                                        .all_series()
                                        .find(|s| s.symbol == *selected_symbol)
                                })
                                .and_then(|s| s.data.last_candle().map(|c| c.close))
                                .unwrap_or(0.0)
                        }
                    } else {
                        app.chart_state.series_manager
                            .all_series()
                            .find(|s| s.symbol == *selected_symbol)
                            .and_then(|s| s.data.last_candle().map(|c| c.close))
                            .unwrap_or(0.0)
                    }
                } else {
                    active_series
                        .and_then(|s| s.data.last_candle().map(|c| c.close))
                        .unwrap_or(0.0)
                }
            };
            
            let (price, total_amount) = match app.trading_state.order_type {
                OrderType::Market => {
                    let total = quantity * current_price;
                    (current_price, total)
                }
                OrderType::Limit => {
                    if let Some(limit_price) = app.trading_state.parse_limit_price() {
                        if limit_price > 0.0 {
                            let total = quantity * limit_price;
                            (limit_price, total)
                        } else {
                            println!("❌ Prix limite invalide");
                            return Task::none();
                        }
                    } else {
                        println!("❌ Prix limite invalide");
                        return Task::none();
                    }
                }
            };
            
            // Récupérer TP et SL (en mode Market, vérifier la checkbox)
            let take_profit = if app.trading_state.order_type == OrderType::Market && !app.trading_state.tp_sl_enabled {
                None
            } else {
                app.trading_state.parse_take_profit()
            };
            let stop_loss = if app.trading_state.order_type == OrderType::Market && !app.trading_state.tp_sl_enabled {
                None
            } else {
                app.trading_state.parse_stop_loss()
            };
            
            println!("📉 Ordre de vente: {} {} à {:.2} USDT (Total: {:.2} USDT)", 
                quantity, symbol, price, total_amount);
            
            match app.trading_state.order_type {
                OrderType::Market => {
                    // En mode démo, simuler l'ordre
                    if app.account_type.is_demo() {
                        // Récupérer le timestamp de la dernière bougie
                        let timestamp = app.chart_state.series_manager
                            .active_series()
                            .next()
                            .and_then(|s| s.data.last_candle().map(|c| c.timestamp));
                        
                        // Essayer de fermer une position existante
                        if let Some(trade) = app.trading_state.trade_history.close_position_with_strategy(
                            &symbol, quantity, price, None, None, timestamp
                        ) {
                            println!("  ✅ Position fermée: Trade #{} (P&L: {:.2} USDT)", trade.id, trade.realized_pnl);
                        } else {
                            // Aucune position à fermer, ouvrir une position short
                            let trade = app.trading_state.trade_history.open_sell_position_with_tp_sl_and_strategy(
                                symbol.clone(),
                                quantity,
                                price,
                                take_profit,
                                stop_loss,
                                None,
                                None,
                                timestamp,
                            );
                            
                            println!("  ✅ Position short ouverte: Trade #{}", trade.id);
                            if take_profit.is_some() || stop_loss.is_some() {
                                println!("  📊 TP: {:?}, SL: {:?}", take_profit, stop_loss);
                            }
                        }
                        
                        // Sauvegarder l'historique
                        if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                            eprintln!("⚠️ Erreur sauvegarde historique trading: {}", e);
                        }
                        
                        // Mettre à jour les informations du compte
                        app.update_account_info();
                    } else {
                        // Mode réel: placer un vrai ordre MARKET SELL via l'API du provider
                        let provider = Arc::clone(&app.binance_provider);
                        let symbol_clone = symbol.clone();
                        let quantity_clone = quantity;
                        
                        // Valider l'ordre avant de le placer
                        // Pour les ordres SELL, on vérifie qu'on a assez de l'asset de base
                        // (cette validation simple vérifie juste le format, la validation réelle
                        // se fera côté Binance qui vérifiera qu'on a assez de l'asset)
                        if quantity_clone <= 0.0 {
                            println!("  ❌ Quantité invalide: {}", quantity_clone);
                            return Task::none();
                        }
                        
                        return Task::perform(
                            async move {
                                place_market_sell_order(&provider, &symbol_clone, quantity_clone)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            crate::app::messages::Message::SellOrderPlaced,
                        );
                    }
                }
                OrderType::Limit => {
                    // En mode démo, créer un ordre en attente
                    if app.account_type.is_demo() {
                        let order = app.trading_state.trade_history.create_pending_order(
                            symbol.clone(),
                            crate::app::data::TradeType::Sell,
                            quantity,
                            price,
                            take_profit,
                            stop_loss,
                        );
                        println!("  ✅ Ordre limit créé: Order #{} (sera exécuté si prix >= {:.2})", 
                            order.id, price);
                        
                        // Vérifier immédiatement si l'ordre peut être exécuté
                        let timestamp = app.chart_state.series_manager
                            .active_series()
                            .next()
                            .and_then(|s| s.data.last_candle().map(|c| c.timestamp));
                        app.trading_state.trade_history.check_and_execute_pending_orders(&symbol, current_price, timestamp);
                        
                        // Sauvegarder l'historique
                        if let Err(e) = app.trading_state.trade_history.save_to_file("paper_trading.json") {
                            eprintln!("⚠️ Erreur sauvegarde historique trading: {}", e);
                        }
                        
                        // Mettre à jour les informations du compte
                        app.update_account_info();
                    } else {
                        // Mode réel: placer un vrai ordre LIMIT SELL via l'API du provider
                        let provider = Arc::clone(&app.binance_provider);
                        let symbol_clone = symbol.clone();
                        let quantity_clone = quantity;
                        let price_clone = price;
                        
                        // Valider l'ordre avant de le placer
                        if quantity_clone <= 0.0 {
                            println!("  ❌ Quantité invalide: {}", quantity_clone);
                            return Task::none();
                        }
                        if price_clone <= 0.0 {
                            println!("  ❌ Prix invalide: {}", price_clone);
                            return Task::none();
                        }
                        
                        return Task::perform(
                            async move {
                                place_limit_sell_order(&provider, &symbol_clone, quantity_clone, price_clone, Some("GTC"))
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            crate::app::messages::Message::SellOrderPlaced,
                        );
                    }
                }
            }
        } else {
            println!("❌ Quantité invalide: {}", quantity);
        }
    } else {
        println!("❌ Quantité invalide: {}", app.trading_state.order_quantity);
    }
    Task::none()
}

/// Gère le résultat du placement d'un ordre d'achat
pub fn handle_buy_order_placed(
    app: &mut ChartApp,
    result: Result<crate::app::trading::api::OrderResponse, String>,
) -> Task<crate::app::messages::Message> {
    match result {
        Ok(order_response) => {
            log::info!("✅ Ordre d'achat placé avec succès! Order ID: {}", order_response.order_id);
            
            // Mettre à jour les informations du compte depuis Binance
            return crate::app::realtime::fetch_account_info(app);
        }
        Err(e) => {
            let error = crate::app::error_handling::AppError::new(
                format!("Erreur lors du placement de l'ordre d'achat"),
                e.clone(),
                crate::app::error_handling::ErrorType::Api,
            )
            .with_source("Trading API".to_string());
            
            error.log();
            app.ui.add_error(error);
        }
    }
    Task::none()
}

/// Gère le résultat du placement d'un ordre de vente
pub fn handle_sell_order_placed(
    app: &mut ChartApp,
    result: Result<crate::app::trading::api::OrderResponse, String>,
) -> Task<crate::app::messages::Message> {
    match result {
        Ok(order_response) => {
            log::info!("✅ Ordre de vente placé avec succès! Order ID: {}", order_response.order_id);
            
            // Mettre à jour les informations du compte depuis Binance
            return crate::app::realtime::fetch_account_info(app);
        }
        Err(e) => {
            let error = crate::app::error_handling::AppError::new(
                format!("Erreur lors du placement de l'ordre de vente"),
                e.clone(),
                crate::app::error_handling::ErrorType::Api,
            )
            .with_source("Trading API".to_string());
            
            error.log();
            app.ui.add_error(error);
        }
    }
    Task::none()
}

/// Gère le toggle du type de compte
pub fn handle_toggle_account_type(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::state::AccountType;
    use crate::app::realtime::fetch_account_info;
    
    // Basculer entre démo et réel
    let was_demo = app.account_type.is_demo();
    let new_type = if was_demo {
        AccountType::Real
    } else {
        AccountType::Demo
    };
    app.account_type.set_account_type(new_type);
    
    // Si on passe du mode paper au mode réel, récupérer les informations du compte
    if was_demo && app.account_type.is_real() {
        // Vérifier que le provider est configuré avec token et secret
        let has_config = app.provider_config
            .active_config()
            .map(|config| {
                config.api_token.is_some() && config.api_secret.is_some()
            })
            .unwrap_or(false);
        
        if has_config {
            println!("🔄 Passage en mode réel : mise à jour des informations du compte...");
            return fetch_account_info(app);
        } else {
            println!("ℹ️ Passage en mode réel : configurez votre provider (API key et secret) pour récupérer les informations du compte.");
        }
    }
    
    Task::none()
}

