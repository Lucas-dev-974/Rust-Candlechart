//! Paramètres configurables des indicateurs techniques

use serde::{Deserialize, Serialize};

/// Méthode de calcul du RSI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RSIMethod {
    /// Méthode standard de Welles Wilder (moyenne mobile lissée)
    Wilder,
    /// Méthode simple (moyenne mobile simple - SMA)
    Simple,
}

impl RSIMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            RSIMethod::Wilder => "Wilder",
            RSIMethod::Simple => "Simple",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Wilder" => Some(RSIMethod::Wilder),
            "Simple" => Some(RSIMethod::Simple),
            _ => None,
        }
    }
}

impl Default for RSIMethod {
    fn default() -> Self {
        RSIMethod::Wilder
    }
}

/// Paramètres pour tous les indicateurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorParams {
    // RSI
    pub rsi_period: usize,
    pub rsi_method: RSIMethod,
    
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
            rsi_method: RSIMethod::Wilder,
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



