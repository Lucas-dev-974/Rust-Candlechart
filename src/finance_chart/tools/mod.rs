//! Module des outils de dessin
//!
//! Contient les types et états pour les outils de dessin (rectangles, lignes)
//! ainsi que le panel d'outils.

pub mod state;
pub mod panel;

// Ré-exports
pub use state::{
    Tool, ToolsState, DrawnRectangle, DrawnHorizontalLine,
    EditMode, EditState, Action, HANDLE_SIZE,
};
pub use panel::{tools_panel, TOOLS_PANEL_WIDTH};

