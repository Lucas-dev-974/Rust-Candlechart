//! Module des axes du graphique
//!
//! Contient les canvas et styles pour les axes X (temps) et Y (prix).

pub mod canvas;
pub mod style;

// Ré-exports
pub use canvas::{x_axis, y_axis, X_AXIS_HEIGHT, Y_AXIS_WIDTH};
pub use style::AxisStyle;

