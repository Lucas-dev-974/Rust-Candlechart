//! Module Volume
//!
//! Contient le rendu graphique et l'axe Y du volume.

pub mod chart;
pub mod axis;

// Ré-exports pour faciliter l'accès
pub use chart::volume_chart;
pub use axis::volume_y_axis;

