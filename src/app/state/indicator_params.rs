//! Paramètres configurables des indicateurs techniques

use serde::{Deserialize, Serialize};

/// Paramètres pour tous les indicateurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorParams {
    // RSI
    pub rsi_period: usize,
    
    // MACD
    pub macd_fast_period: usize,
    pub macd_slow_period: usize,
    pub macd_signal_period: usize,
    
    // Bollinger Bands
    pub bollinger_period: usize,
    pub bollinger_std_dev: f64,
    
    // Moving Average
    pub ma_period: usize,
}

impl Default for IndicatorParams {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            macd_fast_period: 12,
            macd_slow_period: 26,
            macd_signal_period: 9,
            bollinger_period: 20,
            bollinger_std_dev: 2.0,
            ma_period: 20,
        }
    }
}

impl IndicatorParams {
    pub fn new() -> Self {
        Self::default()
    }
}



