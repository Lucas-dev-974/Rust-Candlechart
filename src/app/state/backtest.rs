//! État du backtest
//!
//! Ce module gère l'état du backtest : date de départ, état de lecture, etc.

/// État du backtest
#[derive(Debug, Clone)]
pub struct BacktestState {
    /// Indique si le mode backtest est activé
    pub enabled: bool,
    /// Timestamp de départ sélectionné (None si aucune date n'est sélectionnée)
    pub start_timestamp: Option<i64>,
    /// Indique si le backtest est en cours de lecture
    pub is_playing: bool,
    /// Index actuel dans les bougies (pour la lecture)
    pub current_index: usize,
    /// Index de départ dans les bougies (calculé une fois au démarrage)
    pub start_index: Option<usize>,
    /// Vitesse de lecture (en millisecondes entre chaque bougie)
    pub playback_speed_ms: u64,
    /// ID de la stratégie sélectionnée pour le backtest (None = aucune stratégie)
    pub selected_strategy_id: Option<String>,
}

impl Default for BacktestState {
    fn default() -> Self {
        Self {
            enabled: false,
            start_timestamp: None,
            is_playing: false,
            current_index: 0,
            start_index: None,
            playback_speed_ms: 100, // 100ms par défaut (10 bougies par seconde)
            selected_strategy_id: None,
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
        self.is_playing = true;
        self.current_index = 0;
        // start_index sera calculé dans le handler avec les données de la série
    }
    
    /// Définit l'index de départ calculé
    pub fn set_start_index(&mut self, start_index: usize) {
        self.start_index = Some(start_index);
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
        self.current_index = 0;
        self.start_index = None;
    }
    
    /// Arrête le backtest en gardant la position actuelle (utilisé quand on atteint la fin)
    pub fn stop_at_end(&mut self) {
        self.is_playing = false;
        // Ne pas réinitialiser current_index ni start_index pour garder la position
    }
    
    /// Met à jour l'index actuel
    pub fn update_index(&mut self, index: usize) {
        self.current_index = index;
    }
    
    /// Met à jour la vitesse de lecture
    pub fn set_speed(&mut self, speed_ms: u64) {
        self.playback_speed_ms = speed_ms;
    }
    
    /// Active ou désactive le mode backtest
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        // Si on désactive le backtest, arrêter la lecture
        if !enabled {
            self.is_playing = false;
        }
    }
    
    /// Obtient le timestamp actuel de la bougie en cours de lecture
    /// Retourne None si le backtest n'est pas actif ou si les données ne sont pas disponibles
    pub fn current_candle_timestamp(&self, all_candles: &[crate::finance_chart::core::Candle]) -> Option<i64> {
        if all_candles.is_empty() {
            return None;
        }
        
        if let (Some(start_idx), Some(_start_ts)) = (self.start_index, self.start_timestamp) {
            let current_idx = start_idx + self.current_index;
            if current_idx < all_candles.len() {
                Some(all_candles[current_idx].timestamp)
            } else {
                // Si on dépasse, retourner le timestamp de la dernière bougie
                Some(all_candles[all_candles.len() - 1].timestamp)
            }
        } else if let Some(start_ts) = self.start_timestamp {
            // Si pas d'index mais un timestamp, utiliser le timestamp de départ
            Some(start_ts)
        } else {
            None
        }
    }
}


