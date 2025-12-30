//! État des panneaux latéraux (droite et bas)
//!
//! Gère la visibilité, les dimensions et l'état de redimensionnement des panneaux.

use serde::{Deserialize, Serialize};

/// Taille minimale d'un panneau (juste la poignée visible)
pub const MIN_PANEL_SIZE: f32 = 6.0;

/// Seuil de snap : distance depuis le bord pour déclencher le snap
pub const SNAP_THRESHOLD: f32 = 20.0;

/// État d'un panneau (droite ou bas)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    /// Indique si le panneau est visible
    pub visible: bool,
    /// Dimension actuelle du panneau (largeur pour droite, hauteur pour bas)
    pub size: f32,
    /// Dimension minimale du panneau
    #[serde(skip)]
    pub min_size: f32,
    /// Dimension maximale du panneau
    #[serde(skip)]
    pub max_size: f32,
    /// Indique si le panneau est en cours de redimensionnement
    #[serde(skip)]
    pub is_resizing: bool,
    /// Position de départ du redimensionnement
    #[serde(skip)]
    pub resize_start: Option<f32>,
    /// Indique si le panneau a le focus (les événements du chart sont désactivés)
    #[serde(skip)]
    pub focused: bool,
}

impl PanelState {
    pub fn new(default_size: f32, min_size: f32, max_size: f32) -> Self {
        Self {
            visible: true,
            size: default_size,
            min_size,
            max_size,
            is_resizing: false,
            resize_start: None,
            focused: false,
        }
    }

    /// Démarre le redimensionnement
    pub fn start_resize(&mut self, start_pos: f32) {
        self.is_resizing = true;
        self.resize_start = Some(start_pos);
    }

    /// Met à jour le redimensionnement
    pub fn update_resize(&mut self, current_pos: f32, is_horizontal: bool) {
        if let Some(start) = self.resize_start {
            let delta = if is_horizontal {
                // Pour le panneau de droite, on redimensionne vers la gauche
                start - current_pos
            } else {
                // Pour le panneau du bas, handle en haut : 
                // Glisser vers le haut (current_pos diminue) = agrandir → delta positif
                // Glisser vers le bas (current_pos augmente) = diminuer → delta négatif
                // start - current_pos : si on glisse vers le haut, current_pos < start, donc delta positif
                start - current_pos
            };
            
            let new_size = (self.size + delta).max(self.min_size).min(self.max_size);
            self.size = new_size;
            self.resize_start = Some(current_pos);
        }
    }
    
    /// Applique le snap à la fin du redimensionnement
    /// Si le panneau est proche du minimum, il se snap au minimum
    pub fn apply_snap(&mut self) {
        // Si on est proche du minimum (dans le seuil de snap), snapper au minimum
        if self.size <= self.min_size + SNAP_THRESHOLD && self.size > self.min_size {
            self.size = MIN_PANEL_SIZE;
        }
    }
    
    /// Vérifie si le panneau est réduit au minimum (snappé)
    pub fn is_snapped(&self) -> bool {
        self.size <= MIN_PANEL_SIZE + 1.0 // Petite marge pour éviter les problèmes de précision flottante
    }

    /// Termine le redimensionnement et applique le snap si nécessaire
    pub fn end_resize(&mut self) {
        // Appliquer le snap avant de terminer le redimensionnement
        self.apply_snap();
        self.is_resizing = false;
        self.resize_start = None;
    }

    /// Bascule la visibilité
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Met le panneau en focus
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

/// État de tous les panneaux
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelsState {
    /// Panneau de droite
    pub right: PanelState,
    /// Panneau du bas
    pub bottom: PanelState,
    /// Panneau du volume chart (redimensionnable en hauteur)
    #[serde(default = "default_volume_panel")]
    pub volume: PanelState,
    /// Panneau du RSI chart (redimensionnable en hauteur)
    #[serde(default = "default_rsi_panel")]
    pub rsi: PanelState,
    /// Panneau du MACD chart (redimensionnable en hauteur)
    #[serde(default = "default_macd_panel")]
    pub macd: PanelState,
}

/// Fonction helper pour créer un volume panel par défaut lors de la désérialisation
fn default_volume_panel() -> PanelState {
    use crate::app::utils::constants::VOLUME_CHART_HEIGHT;
    PanelState::new(VOLUME_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0)
}

/// Fonction helper pour créer un RSI panel par défaut lors de la désérialisation
fn default_rsi_panel() -> PanelState {
    use crate::app::utils::constants::RSI_CHART_HEIGHT;
    let mut panel = PanelState::new(RSI_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0);
    panel.visible = false; // Le RSI panel est masqué par défaut
    panel
}

/// Fonction helper pour créer un MACD panel par défaut lors de la désérialisation
fn default_macd_panel() -> PanelState {
    use crate::app::utils::constants::MACD_CHART_HEIGHT;
    let mut panel = PanelState::new(MACD_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0);
    panel.visible = false; // Le MACD panel est masqué par défaut
    panel
}

impl PanelsState {
    pub fn new() -> Self {
        use crate::app::utils::constants::{RIGHT_PANEL_WIDTH, BOTTOM_PANEL_HEIGHT, VOLUME_CHART_HEIGHT, RSI_CHART_HEIGHT, MACD_CHART_HEIGHT};
        let mut rsi_panel = PanelState::new(RSI_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0);
        rsi_panel.visible = false; // Le RSI panel est masqué par défaut
        let mut macd_panel = PanelState::new(MACD_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0);
        macd_panel.visible = false; // Le MACD panel est masqué par défaut
        Self {
            // Taille minimale = juste la poignée (MIN_PANEL_SIZE)
            right: PanelState::new(RIGHT_PANEL_WIDTH, MIN_PANEL_SIZE, 500.0),
            bottom: PanelState::new(BOTTOM_PANEL_HEIGHT, MIN_PANEL_SIZE, 400.0),
            volume: PanelState::new(VOLUME_CHART_HEIGHT, MIN_PANEL_SIZE, 400.0), // Peut être snappé à MIN_PANEL_SIZE
            rsi: rsi_panel, // Peut être snappé à MIN_PANEL_SIZE
            macd: macd_panel, // Peut être snappé à MIN_PANEL_SIZE
        }
    }
    
    
    /// Retourne true si un panneau a le focus
    pub fn has_focused_panel(&self) -> bool {
        self.right.focused || self.bottom.focused || self.volume.focused || self.rsi.focused || self.macd.focused
    }
}

impl Default for PanelsState {
    fn default() -> Self {
        Self::new()
    }
}

