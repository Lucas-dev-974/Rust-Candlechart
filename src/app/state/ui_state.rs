//! État de l'interface utilisateur
//!
//! Ce module regroupe tous les champs liés à l'interface utilisateur
//! (panneaux, sections, drag & drop, menus contextuels).

use crate::app::state::{PanelsState, BottomPanelSectionsState, BottomPanelSection, backtest::BacktestState};

/// État de l'interface utilisateur
#[derive(Debug, Clone)]
pub struct UiState {
    /// État des panneaux latéraux
    pub panels: PanelsState,
    
    /// État des sections du panneau du bas
    pub bottom_panel_sections: BottomPanelSectionsState,
    
    /// État du menu contextuel des sections (section, position globale du curseur)
    pub section_context_menu: Option<(BottomPanelSection, iced::Point)>,
    
    /// Overlay de drag pour les sections (section, position)
    pub drag_overlay: Option<(BottomPanelSection, iced::Point)>,
    
    /// Indique si l'onglet d'indicateurs est ouvert
    pub indicators_panel_open: bool,
    
    /// État du backtest
    pub backtest_state: BacktestState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            panels: PanelsState::new(),
            bottom_panel_sections: BottomPanelSectionsState::new(),
            section_context_menu: None,
            drag_overlay: None,
            indicators_panel_open: false,
            backtest_state: BacktestState::new(),
        }
    }
}

impl UiState {
}




