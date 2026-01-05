//! Handlers pour le backtest

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::messages::Message;

/// Gère la sélection d'une date de départ pour le backtest
pub fn handle_select_backtest_date(app: &mut ChartApp, timestamp: i64) -> Task<Message> {
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
                
                // Démarrer le backtest (réinitialise current_index à 0)
                app.ui.backtest_state.start(start_timestamp);
                app.ui.backtest_state.set_start_index(start_index);
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
    app.ui.backtest_state.stop();
    Task::none()
}

/// Gère un tick du backtest (appelé périodiquement pendant la lecture)
pub fn handle_backtest_tick(app: &mut ChartApp) -> Task<Message> {
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
        let next_index = start_index + current_index;
        
        // Vérifier si on a atteint la fin (avant d'incrémenter)
        if next_index >= candles.len() {
            // Calculer l'index de la dernière bougie valide et le garder
            if candles.len() > 0 && start_index < candles.len() {
                let last_valid_index = candles.len() - 1;
                // Mettre current_index à la position de la dernière bougie
                app.ui.backtest_state.update_index(last_valid_index - start_index);
            }
            // Arrêter le backtest en gardant la position
            app.ui.backtest_state.stop_at_end();
            return Task::none();
        }
        
        // Incrémenter l'index pour passer à la bougie suivante
        app.ui.backtest_state.update_index(current_index + 1);
        
        // Forcer le re-render
        app.render_version += 1;
    } else {
        // Si pas de série active, arrêter le backtest
        app.ui.backtest_state.stop();
    }
    
    Task::none()
}

