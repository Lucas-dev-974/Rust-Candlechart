//! Gestion des sections des panneaux
//!
//! Ce module définit les différentes sections disponibles et leur état
//! dans les panneaux du bas et de droite.

use serde::{Deserialize, Serialize};

/// Sections disponibles dans les panneaux
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
    /// Section pour l'historique des trades
    TradeHistory,
    /// Section pour les stratégies de trading automatisées
    Strategies,
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
            Self::TradeHistory,
            Self::Strategies,
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
            Self::TradeHistory => "Historique",
            Self::Strategies => "Stratégies",
        }
    }
}

/// État des sections des panneaux du bas et de droite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottomPanelSectionsState {
    /// Section actuellement active dans le panneau du bas
    pub active_bottom_section: BottomPanelSection,
    /// Section actuellement active dans le panneau de droite
    pub active_right_section: Option<BottomPanelSection>,
    /// Sections dans le panneau de droite
    pub right_panel_sections: Vec<BottomPanelSection>,
}

impl Default for BottomPanelSectionsState {
    fn default() -> Self {
        Self {
            active_bottom_section: BottomPanelSection::Overview,
            active_right_section: Some(BottomPanelSection::Orders),
            right_panel_sections: vec![
                BottomPanelSection::Orders,
                BottomPanelSection::TradeHistory,
            ],
        }
    }
}

impl BottomPanelSectionsState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Retourne les sections dans le panneau du bas (celles qui ne sont pas dans le panneau de droite)
    pub fn bottom_panel_sections(&self) -> Vec<BottomPanelSection> {
        BottomPanelSection::all()
            .into_iter()
            .filter(|s| !self.right_panel_sections.contains(s))
            .collect()
    }
    
    /// Change la section active du panneau du bas
    pub fn set_active_section(&mut self, section: BottomPanelSection) {
        // Vérifier que la section est bien dans le panneau du bas
        if !self.is_section_in_right_panel(section) {
            self.active_bottom_section = section;
        }
    }
    
    /// Change la section active du panneau de droite
    pub fn set_active_right_section(&mut self, section: BottomPanelSection) {
        // Vérifier que la section est bien dans le panneau de droite
        if self.is_section_in_right_panel(section) {
            self.active_right_section = Some(section);
        }
    }
    
    /// Vérifie si une section est dans le panneau de droite
    pub fn is_section_in_right_panel(&self, section: BottomPanelSection) -> bool {
        self.right_panel_sections.contains(&section)
    }
    
    /// Vérifie si le panneau de droite a des sections
    pub fn has_right_panel_sections(&self) -> bool {
        !self.right_panel_sections.is_empty()
    }
    
    /// Déplace une section vers le panneau de droite
    pub fn move_to_right_panel(&mut self, section: BottomPanelSection) {
        // Retirer de la liste du panneau de droite si déjà présent
        self.right_panel_sections.retain(|&s| s != section);
        // Ajouter à la liste du panneau de droite
        self.right_panel_sections.push(section);
        
        // Si c'était la section active du panneau du bas, changer l'active
        if self.active_bottom_section == section {
            // Trouver une autre section pour le panneau du bas
            if let Some(&first_bottom) = self.bottom_panel_sections().first() {
                self.active_bottom_section = first_bottom;
            }
        }
        
        // Activer cette section dans le panneau de droite
        self.active_right_section = Some(section);
    }
    
    /// Déplace une section vers le panneau du bas
    pub fn move_to_bottom_panel(&mut self, section: BottomPanelSection) {
        // Retirer du panneau de droite
        self.right_panel_sections.retain(|&s| s != section);
        
        // Si c'était la section active du panneau de droite, changer l'active
        if self.active_right_section == Some(section) {
            // Trouver une autre section pour le panneau de droite
            if let Some(&first_right) = self.right_panel_sections.first() {
                self.active_right_section = Some(first_right);
            } else {
                self.active_right_section = None;
            }
        }
        
        // Activer cette section dans le panneau du bas
        self.active_bottom_section = section;
    }
}

