//! Handlers pour le backtest

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::messages::Message;
use crate::app::strategies::strategy::{MarketContext, TradingSignal, TradingMode};
use crate::app::data::OrderType;

/// Sauvegarde les trades du backtest dans un fichier spécifique
fn save_backtest_trades(app: &ChartApp, strategy_id: &str) {
    // Créer le nom du fichier basé sur le nom de la stratégie
    // Nettoyer le nom de la stratégie pour qu'il soit valide comme nom de fichier
    let strategy_name = app.strategy_manager
        .get_strategy(strategy_id)
        .map(|reg| reg.strategy.name())
        .unwrap_or(strategy_id);
    
    // Remplacer les caractères invalides par des underscores
    let safe_name = strategy_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    
    let filename = format!("{}-trades.json", safe_name);
    
    // Sauvegarder l'historique de trades du backtest
    if let Err(e) = app.ui.backtest_state.backtest_trade_history.save_to_file(&filename) {
        eprintln!("⚠️ Erreur sauvegarde trades backtest: {}", e);
    }
}

/// Active ou désactive le mode backtest
pub fn handle_toggle_backtest_enabled(app: &mut ChartApp) -> Task<Message> {
    let new_state = !app.ui.backtest_state.enabled;
    app.ui.backtest_state.set_enabled(new_state);
    Task::none()
}

/// Sélectionne une stratégie pour le backtest
pub fn handle_select_backtest_strategy(app: &mut ChartApp, strategy_id: Option<String>) -> Task<Message> {
    // Vérifier que la stratégie existe si un ID est fourni
    if let Some(ref id) = strategy_id {
        let strategies = app.strategy_manager.get_all();
        if !strategies.iter().any(|(sid, _)| sid == id) {
            eprintln!("⚠️ Stratégie {} introuvable", id);
            return Task::none();
        }
    }
    
    app.ui.backtest_state.selected_strategy_id = strategy_id.clone();
    if let Some(ref id) = strategy_id {
        println!("✅ Stratégie {} sélectionnée pour le backtest", id);
    } else {
        println!("✅ Aucune stratégie sélectionnée pour le backtest");
    }
    Task::none()
}

/// Définit la tête de lecture à la position du clic droit
pub fn handle_set_playhead_mode(app: &mut ChartApp) -> Task<Message> {
    // Ne permettre que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    // Ne pas permettre de redéfinir la position si la lecture est en cours
    if app.ui.backtest_state.is_playing {
        return Task::none();
    }
    
    // Récupérer la position du clic droit (où le menu a été ouvert)
    if let Some(absolute_position) = app.ui.chart_context_menu {
        // Convertir la position absolue en position relative au graphique
        let relative_position = app.chart_state.interaction.absolute_to_relative(absolute_position);
        
        // Convertir la position X en timestamp
        let viewport = &app.chart_state.viewport;
        let timestamp = viewport.time_scale().x_to_time(relative_position.x);
        
        // Définir la tête de lecture
        app.ui.backtest_state.start_timestamp = Some(timestamp);
        app.ui.backtest_state.current_index = 0;
        app.ui.backtest_state.start_index = None;
    }
    
    // Fermer le menu contextuel
    app.ui.chart_context_menu = None;
    
    Task::none()
}

/// Démarre le drag de la tête de lecture
pub fn handle_start_drag_playhead(app: &mut ChartApp, _position: iced::Point) -> Task<Message> {
    // Ne permettre que si le backtest est activé et pas en lecture
    if !app.ui.backtest_state.enabled || app.ui.backtest_state.is_playing {
        return Task::none();
    }
    
    // Vérifier qu'on a un timestamp de départ
    if app.ui.backtest_state.start_timestamp.is_none() {
        return Task::none();
    }
    
    // Activer le mode drag
    app.ui.backtest_state.dragging_playhead = true;
    
    Task::none()
}

/// Met à jour la position de la tête de lecture pendant le drag
pub fn handle_update_drag_playhead(app: &mut ChartApp, position: iced::Point) -> Task<Message> {
    // Ne permettre que si on est en train de drag
    if !app.ui.backtest_state.dragging_playhead {
        return Task::none();
    }
    
    // Convertir la position absolue en position relative au graphique
    let relative_position = app.chart_state.interaction.absolute_to_relative(position);
    
    // Convertir la position X en timestamp
    let viewport = &app.chart_state.viewport;
    let timestamp = viewport.time_scale().x_to_time(relative_position.x);
    
    // Mettre à jour le timestamp (mais ne pas réinitialiser les index tant qu'on n'a pas fini)
    app.ui.backtest_state.start_timestamp = Some(timestamp);
    
    Task::none()
}

/// Termine le drag de la tête de lecture
pub fn handle_end_drag_playhead(app: &mut ChartApp) -> Task<Message> {
    // Ne permettre que si on est en train de drag
    if !app.ui.backtest_state.dragging_playhead {
        return Task::none();
    }
    
    // Désactiver le mode drag
    app.ui.backtest_state.dragging_playhead = false;
    
    // Réinitialiser les index pour que la barre se positionne correctement
    app.ui.backtest_state.current_index = 0;
    app.ui.backtest_state.start_index = None;
    
    Task::none()
}

/// Gère la sélection d'une date de départ pour le backtest
pub fn handle_select_backtest_date(app: &mut ChartApp, timestamp: i64) -> Task<Message> {
    // Ne permettre la sélection que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    // Ne pas permettre de redéfinir la position si la lecture est en cours
    if !app.ui.backtest_state.is_playing {
        // Mettre à jour le timestamp de départ
        app.ui.backtest_state.start_timestamp = Some(timestamp);
        
        // Réinitialiser les index pour que la barre se positionne sur la nouvelle date
        app.ui.backtest_state.current_index = 0;
        app.ui.backtest_state.start_index = None;
    }
    
    Task::none()
}

/// Démarre la lecture du backtest
pub fn handle_start_backtest(app: &mut ChartApp) -> Task<Message> {
    // Ne permettre le démarrage que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    if let Some(start_timestamp) = app.ui.backtest_state.start_timestamp {
        // Récupérer la série active pour calculer l'index de départ
        let active_series = app.chart_state.series_manager
            .active_series()
            .next();
        
        if let Some(series) = active_series {
            let candles = series.data.all_candles();
            
            // Vérifier si on reprend depuis une pause ou si on démarre un nouveau backtest
            let is_resuming = app.ui.backtest_state.start_index.is_some() 
                && !app.ui.backtest_state.is_playing;
            
            if is_resuming {
                // Reprendre depuis une pause : ne pas réinitialiser current_index
                app.ui.backtest_state.resume();
            } else {
                // Nouveau démarrage : calculer l'index de départ et réinitialiser
                let start_index = candles.iter()
                    .position(|c| c.timestamp >= start_timestamp)
                    .unwrap_or(0);
                
                // Vérifier que l'index de départ est valide
                if start_index >= candles.len() {
                    // Si l'index est invalide (timestamp après toutes les bougies), ne pas démarrer
                    return Task::none();
                }
                
                // Initialiser le capital du backtest avec le capital actuel du compte
                let initial_capital = app.account_info.total_balance;
                app.ui.backtest_state.reset_with_capital(initial_capital);
                
                // Démarrer le backtest (réinitialise current_index à 0)
                app.ui.backtest_state.start(start_timestamp);
                app.ui.backtest_state.set_start_index(start_index);
                
                println!("📊 Backtest démarré avec capital initial: {:.2} USDT", initial_capital);
            }
            
            // La subscription sera automatiquement mise à jour lors du prochain cycle
            Task::none()
        } else {
            Task::none()
        }
    } else {
        Task::none()
    }
}

/// Met en pause la lecture du backtest
pub fn handle_pause_backtest(app: &mut ChartApp) -> Task<Message> {
    // Ne permettre la pause que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    if app.ui.backtest_state.is_playing {
        app.ui.backtest_state.pause();
    } else {
        // Si en pause, reprendre la lecture
        app.ui.backtest_state.resume();
    }
    Task::none()
}

/// Arrête la lecture du backtest
pub fn handle_stop_backtest(app: &mut ChartApp) -> Task<Message> {
    // Ne permettre l'arrêt que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    // Sauvegarder les trades du backtest avant d'arrêter
    if let Some(ref strategy_id) = app.ui.backtest_state.selected_strategy_id {
        save_backtest_trades(app, strategy_id);
    }
    
    app.ui.backtest_state.stop();
    Task::none()
}

/// Gère un tick du backtest (appelé périodiquement pendant la lecture)
pub fn handle_backtest_tick(app: &mut ChartApp) -> Task<Message> {
    // Ne traiter les ticks que si le backtest est activé
    if !app.ui.backtest_state.enabled {
        return Task::none();
    }
    
    if !app.ui.backtest_state.is_playing {
        return Task::none();
    }
    
    // Récupérer la série active
    let active_series = app.chart_state.series_manager
        .active_series()
        .next();
    
    if let Some(series) = active_series {
        let candles = series.data.all_candles();
        
        // Utiliser l'index de départ stocké, ou le recalculer si nécessaire
        let start_index = if let Some(stored_index) = app.ui.backtest_state.start_index {
            // Vérifier que l'index stocké est toujours valide
            if stored_index < candles.len() {
                stored_index
            } else {
                // Si l'index n'est plus valide (série changée ou données modifiées), recalculer
                let start_timestamp = app.ui.backtest_state.start_timestamp.unwrap_or(0);
                candles.iter()
                    .position(|c| c.timestamp >= start_timestamp)
                    .unwrap_or(0)
            }
        } else {
            // Si pas d'index stocké, recalculer (ne devrait pas arriver normalement)
            let start_timestamp = app.ui.backtest_state.start_timestamp.unwrap_or(0);
            candles.iter()
                .position(|c| c.timestamp >= start_timestamp)
                .unwrap_or(0)
        };
        
        // Mettre à jour l'index stocké si on l'a recalculé
        let needs_update = match app.ui.backtest_state.start_index {
            Some(stored) => stored != start_index,
            None => true,
        };
        if needs_update {
            app.ui.backtest_state.set_start_index(start_index);
        }
        
        let current_index = app.ui.backtest_state.current_index;
        let current_candle_index = start_index + current_index;
        
        // Vérifier si on a atteint la fin
        if current_candle_index >= candles.len() {
            // Calculer l'index de la dernière bougie valide et le garder
            if candles.len() > 0 && start_index < candles.len() {
                let last_valid_index = candles.len() - 1;
                // Mettre current_index à la position de la dernière bougie
                app.ui.backtest_state.update_index(last_valid_index - start_index);
            }
            
            // Sauvegarder les trades du backtest avant d'arrêter
            if let Some(ref strategy_id) = app.ui.backtest_state.selected_strategy_id {
                save_backtest_trades(app, strategy_id);
            }
            
            // Arrêter le backtest en gardant la position
            app.ui.backtest_state.stop_at_end();
            return Task::none();
        }
        
        // Obtenir la bougie actuelle pour vérifier les TP/SL
        let current_candle = &candles[current_candle_index];
        let current_price = current_candle.close;
        
        // Vérifier et exécuter les TP/SL des positions ouvertes pour ce symbole dans le backtest
        // Utiliser le trade_history du backtest, pas celui du compte principal
        let had_trades_before = !app.ui.backtest_state.backtest_trade_history.trades.is_empty();
        
        app.ui.backtest_state.backtest_trade_history.check_take_profit_stop_loss(
            &series.symbol, 
            current_price, 
            Some(current_candle.timestamp)
        );
        
        // Sauvegarder si des trades ont été fermés par TP/SL
        let has_trades_after = !app.ui.backtest_state.backtest_trade_history.trades.is_empty();
        if has_trades_after && had_trades_before {
            if let Some(ref strategy_id) = app.ui.backtest_state.selected_strategy_id {
                save_backtest_trades(app, strategy_id);
            }
        }
        
        // Exécuter la stratégie sélectionnée si elle existe (sur la bougie actuelle)
        if let Some(ref strategy_id) = app.ui.backtest_state.selected_strategy_id {
            // Cloner les données nécessaires pour éviter les problèmes d'emprunt
            let strategy_id_clone = strategy_id.clone();
            let series_clone = series.clone();
            execute_backtest_strategy(app, &strategy_id_clone, &series_clone, current_candle_index);
        }
        
        // Incrémenter l'index pour passer à la bougie suivante (après avoir traité la bougie actuelle)
        app.ui.backtest_state.update_index(current_index + 1);
        
        // Forcer le re-render
        app.render_version += 1;
    } else {
        // Si pas de série active, arrêter le backtest
        app.ui.backtest_state.stop();
    }
    
    Task::none()
}

/// Exécute une stratégie spécifique dans le contexte du backtest
fn execute_backtest_strategy(
    app: &mut ChartApp,
    strategy_id: &str,
    series: &crate::finance_chart::core::SeriesData,
    current_candle_index: usize,
) {
    // Récupérer la stratégie
    let Some(reg) = app.strategy_manager.get_strategy(strategy_id) else {
        return;
    };
    
    let candles = series.data.all_candles();
    
    // Vérifier que l'index est valide
    if current_candle_index >= candles.len() {
        return;
    }
    
    // Créer le contexte de marché avec les bougies jusqu'à l'index actuel
    // (pour simuler l'état du marché au moment du backtest)
    let historical_candles: Vec<_> = candles[..=current_candle_index].to_vec();
    let current_candle = candles[current_candle_index].clone();
    let current_price = current_candle.close;
    let current_volume = current_candle.volume;
    
    let context = MarketContext {
        symbol: series.symbol.clone(),
        series_id: series.id.clone(),
        current_candle: current_candle.clone(),
        candles: historical_candles,
        current_price,
        current_volume,
    };
    
    // Évaluer la stratégie
    let result = reg.strategy.evaluate(&context);
    let strategy_name = reg.strategy.name().to_string();
    let trading_mode = reg.trading_mode;
    
    // Filtrer les signaux selon le mode de trading
    let signal = match (&result.signal, trading_mode) {
        (TradingSignal::Buy { .. }, TradingMode::SellOnly) => {
            return; // Ignorer les signaux d'achat si mode vente uniquement
        }
        (TradingSignal::Sell { .. }, TradingMode::BuyOnly) => {
            return; // Ignorer les signaux de vente si mode achat uniquement
        }
        _ => result.signal.clone(),
    };
    
    // Exécuter le signal uniquement en mode demo
    if !app.account_type.is_demo() {
        return;
    }
    
    match signal {
        TradingSignal::Buy { quantity, order_type, take_profit, stop_loss, .. } => {
            println!("🤖 [Backtest - {}] Signal d'achat: {} (confiance: {:.2}%)", 
                strategy_id, result.reason, result.confidence * 100.0);
            
            let price = match order_type {
                OrderType::Market => current_price,
                OrderType::Limit => current_price, // Simplifié pour le backtest
            };
            
            let position = app.ui.backtest_state.backtest_trade_history.open_buy_position_with_tp_sl_and_strategy(
                series.symbol.clone(),
                quantity,
                price,
                take_profit,
                stop_loss,
                Some(strategy_id.to_string()),
                Some(strategy_name.clone()),
                Some(current_candle.timestamp),
            );
            
            println!("  ✅ Position ouverte (backtest): Trade #{}", position.id);
            
            // Sauvegarder les trades du backtest dans un fichier spécifique
            save_backtest_trades(app, strategy_id);
            
            // Ne pas sauvegarder dans paper_trading.json ni mettre à jour le compte principal
            // Le backtest utilise son propre trade_history isolé
        }
        TradingSignal::Sell { quantity, order_type, take_profit, stop_loss, .. } => {
            println!("🤖 [Backtest - {}] Signal de vente: {} (confiance: {:.2}%)", 
                strategy_id, result.reason, result.confidence * 100.0);
            
            let price = match order_type {
                OrderType::Market => current_price,
                OrderType::Limit => current_price, // Simplifié pour le backtest
            };
            
            // Chercher une position ouverte pour ce symbole dans le backtest
            let open_positions: Vec<_> = app.ui.backtest_state.backtest_trade_history.open_positions
                .iter()
                .filter(|p| p.symbol == series.symbol)
                .collect();
            
            if open_positions.first().is_some() {
                // Fermer la position existante
                let closed_position = app.ui.backtest_state.backtest_trade_history.close_position_with_strategy(
                    &series.symbol,
                    quantity,
                    price,
                    Some(strategy_id.to_string()),
                    Some(strategy_name.clone()),
                    Some(current_candle.timestamp),
                );
                
                if let Some(closed) = closed_position {
                    println!("  ✅ Position fermée (backtest): Trade #{}", closed.id);
                    
                    // Sauvegarder les trades du backtest dans un fichier spécifique
                    save_backtest_trades(app, strategy_id);
                    
                    // Ne pas sauvegarder dans paper_trading.json ni mettre à jour le compte principal
                    // Le backtest utilise son propre trade_history isolé
                }
            } else {
                // Ouvrir une nouvelle position de vente (short)
                let position = app.ui.backtest_state.backtest_trade_history.open_sell_position_with_tp_sl_and_strategy(
                    series.symbol.clone(),
                    quantity,
                    price,
                    take_profit,
                    stop_loss,
                    Some(strategy_id.to_string()),
                    Some(strategy_name.clone()),
                    Some(current_candle.timestamp),
                );
                
                println!("  ✅ Position short ouverte (backtest): Trade #{}", position.id);
                
                // Sauvegarder les trades du backtest dans un fichier spécifique
                save_backtest_trades(app, strategy_id);
                
                // Ne pas sauvegarder dans paper_trading.json ni mettre à jour le compte principal
                // Le backtest utilise son propre trade_history isolé
            }
        }
        TradingSignal::Hold => {
            // Ne rien faire
        }
    }
}

