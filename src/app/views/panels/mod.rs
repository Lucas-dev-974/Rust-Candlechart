//! Panneaux latéraux (droite et bas)

mod indicators;
mod right;
mod bottom;
mod sections;

// Réexporter les fonctions publiques principales
pub use indicators::build_indicator_panels;
pub use right::{view_right_panel, section_context_menu};
pub use bottom::view_bottom_panel;



