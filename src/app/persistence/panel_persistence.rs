//! Persistance de l'état des panneaux
//!
//! Ce module gère la sauvegarde et le chargement de l'état des panneaux
//! (visibilité, taille, section active) dans un fichier JSON.

use serde::{Deserialize, Serialize};
use crate::app::state::{PanelsState, BottomPanelSection};

/// État complet des panneaux à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPersistenceState {
    /// État des panneaux (droite et bas)
    pub panels: PanelsState,
    /// Section active du panneau du bas
    pub active_bottom_section: BottomPanelSection,
    /// Section active du panneau de droite
    #[serde(default)]
    pub active_right_section: Option<BottomPanelSection>,
    /// Sections dans le panneau de droite
    #[serde(default)]
    pub right_panel_sections: Vec<BottomPanelSection>,
}

impl PanelPersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let state: PanelPersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for PanelPersistenceState {
    fn default() -> Self {
        Self {
            panels: PanelsState::new(),
            active_bottom_section: BottomPanelSection::Overview,
            active_right_section: None,
            right_panel_sections: Vec::new(),
        }
    }
}

