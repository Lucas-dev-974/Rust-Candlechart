//! Fonctions de scaling pour le MACD
//!
//! Ce module contient les fonctions pour convertir les valeurs MACD en coordonnées Y
//! et vice versa, en utilisant une plage symétrique autour de zéro.

use crate::finance_chart::render::calculate_nice_step;

/// Configuration de scaling pour le MACD
pub struct MacdScaling {
    /// Valeur minimale symétrique
    pub symmetric_min: f64,
    /// Valeur maximale symétrique
    pub symmetric_max: f64,
    /// Plage totale
    pub macd_range: f64,
    /// Hauteur du graphique
    pub height: f32,
}

impl MacdScaling {
    /// Crée un nouveau scaling MACD à partir d'une plage de valeurs
    ///
    /// # Arguments
    /// * `min_macd` - Valeur MACD minimale
    /// * `max_macd` - Valeur MACD maximale
    /// * `height` - Hauteur du graphique en pixels
    ///
    /// # Retourne
    /// Un nouveau `MacdScaling` avec une plage symétrique autour de zéro
    pub fn new(min_macd: f64, max_macd: f64, height: f32) -> Self {
        // Créer une plage symétrique autour de zéro pour que le centre reste fixe
        let abs_max = min_macd.abs().max(max_macd.abs());
        let symmetric_min = -abs_max;
        let symmetric_max = abs_max;
        let macd_range = (symmetric_max - symmetric_min).max(0.0001);

        Self {
            symmetric_min,
            symmetric_max,
            macd_range,
            height,
        }
    }

    /// Convertit une valeur MACD en coordonnée Y
    pub fn macd_to_y(&self, value: f64) -> f32 {
        let normalized = (value - self.symmetric_min) / self.macd_range;
        self.height * (1.0 - normalized as f32)
    }

    /// Convertit une coordonnée Y en valeur MACD
    pub fn y_to_macd(&self, y: f32) -> f64 {
        let normalized = 1.0 - (y / self.height);
        self.symmetric_min + (normalized as f64 * self.macd_range)
    }

    /// Retourne la position Y de la ligne zéro (toujours au centre)
    pub fn zero_y(&self) -> f32 {
        self.height / 2.0
    }

    /// Calcule le pas pour les niveaux MACD à afficher
    pub fn calculate_step(&self) -> f64 {
        calculate_nice_step(self.macd_range)
    }

    /// Retourne le premier niveau MACD à afficher
    pub fn first_level(&self) -> f64 {
        let step = self.calculate_step();
        (self.symmetric_min / step).ceil() * step
    }
}

