//! État des panneaux latéraux (droite et bas)
//!
//! Gère la visibilité, les dimensions et l'état de redimensionnement des panneaux.

/// Taille minimale d'un panneau (juste la poignée visible)
pub const MIN_PANEL_SIZE: f32 = 6.0;

/// Seuil de snap : distance depuis le bord pour déclencher le snap
pub const SNAP_THRESHOLD: f32 = 20.0;

/// État d'un panneau (droite ou bas)
#[derive(Debug, Clone)]
pub struct PanelState {
    /// Indique si le panneau est visible
    pub visible: bool,
    /// Dimension actuelle du panneau (largeur pour droite, hauteur pour bas)
    pub size: f32,
    /// Dimension minimale du panneau
    pub min_size: f32,
    /// Dimension maximale du panneau
    pub max_size: f32,
    /// Indique si le panneau est en cours de redimensionnement
    pub is_resizing: bool,
    /// Position de départ du redimensionnement
    pub resize_start: Option<f32>,
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
}

/// État de tous les panneaux
#[derive(Debug, Clone)]
pub struct PanelsState {
    /// Panneau de droite
    pub right: PanelState,
    /// Panneau du bas
    pub bottom: PanelState,
}

impl PanelsState {
    pub fn new() -> Self {
        use crate::app::constants::{RIGHT_PANEL_WIDTH, BOTTOM_PANEL_HEIGHT};
        Self {
            // Taille minimale = juste la poignée (MIN_PANEL_SIZE)
            right: PanelState::new(RIGHT_PANEL_WIDTH, MIN_PANEL_SIZE, 500.0),
            bottom: PanelState::new(BOTTOM_PANEL_HEIGHT, MIN_PANEL_SIZE, 400.0),
        }
    }
}

impl Default for PanelsState {
    fn default() -> Self {
        Self::new()
    }
}

