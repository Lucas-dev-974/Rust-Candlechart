//! Module MACD (Moving Average Convergence Divergence)
//!
//! Contient les calculs, le rendu graphique, l'axe Y et le scaling du MACD.

pub mod calc;
pub mod chart;
pub mod axis;
pub mod data;
pub mod scaling;
pub mod snapshot;

// Ré-exports pour faciliter l'accès
pub use calc::MacdValue;
pub use chart::macd_chart;
pub use axis::macd_y_axis;
pub use data::calculate_all_macd_values;

