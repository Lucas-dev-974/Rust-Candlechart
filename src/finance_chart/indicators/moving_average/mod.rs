//! Module de la moyenne mobile (Simple Moving Average - SMA)
//!
//! La moyenne mobile est un indicateur technique qui affiche
//! la moyenne arithmétique des prix de clôture sur une période donnée.

pub mod calc;
pub mod data;

pub use data::{
    calculate_ma_data,
    calculate_all_ma_values,
};

