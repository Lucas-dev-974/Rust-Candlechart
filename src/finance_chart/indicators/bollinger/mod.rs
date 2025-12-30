//! Module des bandes de Bollinger
//!
//! Les bandes de Bollinger sont un indicateur technique qui affiche
//! une bande moyenne (SMA) avec des bandes supérieure et inférieure
//! basées sur l'écart-type.

pub mod calc;
pub mod data;

pub use calc::{calculate_bollinger_bands, BollingerValue, BOLLINGER_PERIOD, BOLLINGER_STD_DEV};
pub use data::{
    calculate_bollinger_data,
    calculate_all_bollinger_values,
    calculate_bollinger_range,
    get_last_bollinger_value,
};


