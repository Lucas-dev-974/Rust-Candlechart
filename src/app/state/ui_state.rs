//! État de l'interface utilisateur
//!
//! Ce module regroupe tous les champs liés à l'interface utilisateur
//! (panneaux, sections, menus contextuels).

use crate::app::state::{PanelsState, BottomPanelSectionsState, BottomPanelSection, backtest::BacktestState};
use crate::app::error_handling::AppError;
use super::notifications::NotificationManager;

/// État de l'interface utilisateur
#[derive(Debug, Clone)]
pub struct UiState {
    /// État des panneaux latéraux
    pub panels: PanelsState,
    
    /// État des sections du panneau du bas
    pub bottom_panel_sections: BottomPanelSectionsState,
    
    /// État du menu contextuel des sections (section, position globale du curseur)
    pub section_context_menu: Option<(BottomPanelSection, iced::Point)>,
    
    /// État du menu contextuel du graphique (position globale du curseur)
    pub chart_context_menu: Option<iced::Point>,
    
    /// Indique si l'onglet d'indicateurs est ouvert
    pub indicators_panel_open: bool,
    
    /// État du backtest
    pub backtest_state: BacktestState,
    
    /// Messages d'erreur à afficher à l'utilisateur (déprécié, utiliser notifications)
    #[deprecated(note = "Utiliser notifications à la place")]
    pub error_messages: Vec<AppError>,
    
    /// Gestionnaire de notifications
    pub notifications: NotificationManager,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            panels: PanelsState::new(),
            bottom_panel_sections: BottomPanelSectionsState::new(),
            section_context_menu: None,
            chart_context_menu: None,
            indicators_panel_open: false,
            backtest_state: BacktestState::new(),
            error_messages: Vec::new(),
            notifications: NotificationManager::new(),
        }
    }
}

impl UiState {
    /// Ajoute un message d'erreur à afficher (déprécié, utiliser notifications)
    #[deprecated(note = "Utiliser notifications.add_error() à la place")]
    pub fn add_error(&mut self, error: AppError) {
        // Limiter à 5 messages d'erreur maximum
        if self.error_messages.len() >= 5 {
            self.error_messages.remove(0);
        }
        self.error_messages.push(error.clone());
        
        // Ajouter aussi dans le système de notifications
        self.notifications.add_error(error);
    }

    /// Supprime un message d'erreur à l'index donné (déprécié)
    #[deprecated(note = "Utiliser notifications.remove() à la place")]
    pub fn remove_error(&mut self, index: usize) {
        if index < self.error_messages.len() {
            self.error_messages.remove(index);
        }
    }

    /// Supprime tous les messages d'erreur (déprécié)
    #[deprecated(note = "Utiliser notifications.clear() à la place")]
    pub fn clear_errors(&mut self) {
        self.error_messages.clear();
    }
    
    /// Met à jour les notifications (supprime les expirées)
    pub fn update_notifications(&mut self) -> Vec<usize> {
        self.notifications.update()
    }
}




