//! État du backtest
//!
//! Ce module gère l'état du backtest : date de départ, état de lecture, etc.

use crate::app::data::TradeHistory;

/// État du backtest
#[derive(Debug, Clone)]
pub struct BacktestState {
    /// Indique si le mode backtest est activé
    pub enabled: bool,
    /// Timestamp de départ sélectionné (None si aucune date n'est sélectionnée)
    pub start_timestamp: Option<i64>,
    /// Timestamp actuel de la bougie en cours de lecture (None si pas encore démarré)
    pub current_timestamp: Option<i64>,
    /// Indique si le backtest est en cours de lecture
    pub is_playing: bool,
    /// Vitesse de lecture (en millisecondes entre chaque bougie)
    pub playback_speed_ms: u64,
    /// ID de la stratégie sélectionnée pour le backtest (None = aucune stratégie)
    pub selected_strategy_id: Option<String>,
    /// Historique de trades séparé pour le backtest (n'affecte pas le compte principal)
    pub backtest_trade_history: TradeHistory,
    /// Capital initial du backtest (utilisé pour calculer les performances)
    pub initial_capital: f64,
    /// Indique si on est en train de déplacer la tête de lecture par drag
    pub dragging_playhead: bool,
}

impl Default for BacktestState {
    fn default() -> Self {
        Self {
            enabled: false,
            start_timestamp: None,
            current_timestamp: None,
            is_playing: false,
            playback_speed_ms: 100, // 100ms par défaut (10 bougies par seconde)
            selected_strategy_id: None,
            backtest_trade_history: TradeHistory::new(),
            initial_capital: 10000.0, // Capital par défaut pour le backtest
            dragging_playhead: false,
        }
    }
}

impl BacktestState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Démarre le backtest depuis le timestamp sélectionné
    pub fn start(&mut self, start_timestamp: i64) {
        self.start_timestamp = Some(start_timestamp);
        self.current_timestamp = Some(start_timestamp);
        self.is_playing = true;
        // Réinitialiser l'historique de trades du backtest
        self.backtest_trade_history = TradeHistory::new();
    }
    
    /// Réinitialise le backtest avec un capital initial
    pub fn reset_with_capital(&mut self, initial_capital: f64) {
        self.backtest_trade_history = TradeHistory::new();
        self.initial_capital = initial_capital;
        // Réinitialiser le timestamp actuel au timestamp de départ
        self.current_timestamp = self.start_timestamp;
    }
    
    /// Met en pause le backtest
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
    
    /// Reprend le backtest
    pub fn resume(&mut self) {
        self.is_playing = true;
    }
    
    /// Arrête le backtest
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_timestamp = None;
        self.dragging_playhead = false; // Réinitialiser le drag si actif
        // Optionnel : on peut garder l'historique pour afficher les résultats
        // ou le réinitialiser : self.backtest_trade_history = TradeHistory::new();
    }
    
    /// Arrête le backtest en gardant la position actuelle (utilisé quand on atteint la fin)
    pub fn stop_at_end(&mut self) {
        self.is_playing = false;
        // Ne pas réinitialiser current_timestamp pour garder la position
    }
    
    /// Met à jour le timestamp actuel
    pub fn update_timestamp(&mut self, timestamp: i64) {
        self.current_timestamp = Some(timestamp);
    }
    
    /// Active ou désactive le mode backtest
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        // Si on désactive le backtest, arrêter la lecture et réinitialiser le drag
        if !enabled {
            self.is_playing = false;
            self.dragging_playhead = false;
        }
    }
    
    /// Obtient le timestamp actuel de la bougie en cours de lecture
    /// Retourne None si le backtest n'est pas actif
    pub fn current_candle_timestamp(&self) -> Option<i64> {
        // Utiliser current_timestamp s'il est défini, sinon fallback sur start_timestamp
        self.current_timestamp.or(self.start_timestamp)
    }
    
    /// Calcule les statistiques du backtest
    pub fn calculate_stats(&self, symbol: &str, current_price: f64) -> BacktestStats {
        let total_realized_pnl = self.backtest_trade_history.total_realized_pnl();
        let total_unrealized_pnl = self.backtest_trade_history.total_unrealized_pnl(symbol, current_price);
        
        // Capital final = capital initial + P&L réalisé + P&L non réalisé
        let final_capital = self.initial_capital + total_realized_pnl + total_unrealized_pnl;
        
        // P&L total (réalisé + non réalisé)
        let total_pnl = total_realized_pnl + total_unrealized_pnl;
        
        // Pourcentage de rendement
        let return_percentage = if self.initial_capital > 0.0 {
            (total_pnl / self.initial_capital) * 100.0
        } else {
            0.0
        };
        
        BacktestStats {
            initial_capital: self.initial_capital,
            final_capital,
            total_realized_pnl,
            total_unrealized_pnl,
            total_pnl,
            return_percentage,
            total_trades: self.backtest_trade_history.trades.len(),
            open_positions: self.backtest_trade_history.open_positions_count(),
        }
    }
}

/// Statistiques du backtest
#[derive(Debug, Clone)]
pub struct BacktestStats {
    /// Capital initial
    pub initial_capital: f64,
    /// Capital final
    pub final_capital: f64,
    /// P&L réalisé (sur positions fermées)
    pub total_realized_pnl: f64,
    /// P&L non réalisé (sur positions ouvertes)
    pub total_unrealized_pnl: f64,
    /// P&L total (réalisé + non réalisé)
    pub total_pnl: f64,
    /// Pourcentage de rendement
    pub return_percentage: f64,
    /// Nombre total de trades
    pub total_trades: usize,
    /// Nombre de positions ouvertes
    pub open_positions: u32,
}


