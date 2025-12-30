//! État des indicateurs techniques
//!
//! Ce module regroupe tous les champs liés aux indicateurs techniques
//! (activation, paramètres, etc.).

use crate::app::state::IndicatorParams;

/// État des indicateurs techniques
#[derive(Debug, Clone)]
pub struct IndicatorState {
    /// Indique si les bandes de Bollinger sont activées
    pub bollinger_bands_enabled: bool,
    
    /// Indique si la moyenne mobile est activée
    pub moving_average_enabled: bool,
    
    /// Paramètres configurables des indicateurs
    pub params: IndicatorParams,
}

impl Default for IndicatorState {
    fn default() -> Self {
        Self {
            bollinger_bands_enabled: false,
            moving_average_enabled: false,
            params: IndicatorParams::new(),
        }
    }
}

impl IndicatorState {
    pub fn new() -> Self {
        Self::default()
    }
}



