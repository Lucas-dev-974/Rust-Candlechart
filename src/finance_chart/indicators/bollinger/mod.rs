//! Module des bandes de Bollinger
//!
//! Les bandes de Bollinger sont un indicateur technique qui affiche
//! une bande moyenne (SMA) avec des bandes supérieure et inférieure
//! basées sur l'écart-type.

pub mod calc;
pub mod data;

pub use calc::BollingerValue;
pub use data::{
    calculate_bollinger_data,
    calculate_all_bollinger_values,
};


