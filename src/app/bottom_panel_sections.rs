//! Gestion des sections du panneau du bas
//!
//! Ce module définit les différentes sections disponibles dans le panneau du bas
//! et leur état.

use serde::{Deserialize, Serialize};

/// Sections disponibles dans le panneau du bas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BottomPanelSection {
    /// Section par défaut (statistiques/informations)
    Overview,
    /// Section pour les logs
    Logs,
    /// Section pour les indicateurs techniques
    Indicators,
    /// Section pour les ordres/trades
    Orders,
    /// Section pour le compte
    Account,
}

impl BottomPanelSection {
    /// Retourne toutes les sections disponibles
    pub fn all() -> Vec<Self> {
        vec![
            Self::Overview,
            Self::Logs,
            Self::Indicators,
            Self::Orders,
            Self::Account,
        ]
    }
    
    /// Retourne le nom d'affichage de la section
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Overview => "Vue d'ensemble",
            Self::Logs => "Logs",
            Self::Indicators => "Indicateurs",
            Self::Orders => "Ordres",
            Self::Account => "Compte",
        }
    }
}

/// État des sections du panneau du bas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottomPanelSectionsState {
    /// Section actuellement active
    pub active_section: BottomPanelSection,
}

impl Default for BottomPanelSectionsState {
    fn default() -> Self {
        Self {
            active_section: BottomPanelSection::Overview,
        }
    }
}

impl BottomPanelSectionsState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Change la section active
    pub fn set_active_section(&mut self, section: BottomPanelSection) {
        self.active_section = section;
    }
}

