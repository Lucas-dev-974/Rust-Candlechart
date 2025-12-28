//! Snapshot léger pour le rendu de l'axe MACD
//!
//! Ce module fournit une structure légère contenant uniquement les données
//! nécessaires au rendu de l'axe MACD, évitant de cloner le ChartState complet.

use super::calc::MacdValue;

/// Snapshot léger pour le rendu de l'axe MACD
/// 
/// Contient uniquement les données pré-calculées nécessaires au dessin,
/// évitant ainsi les clones coûteux du ChartState complet.
#[derive(Debug, Clone)]
pub struct MacdAxisSnapshot {
    /// Valeurs MACD visibles (slice extrait)
    pub visible_macd: Vec<Option<MacdValue>>,
    /// Valeur minimale du MACD pour le scaling
    pub min_macd: f64,
    /// Valeur maximale du MACD pour le scaling
    pub max_macd: f64,
}

impl MacdAxisSnapshot {
    /// Crée un nouveau snapshot avec les données pré-calculées
    pub fn new(
        visible_macd: Vec<Option<MacdValue>>,
        min_macd: f64,
        max_macd: f64,
    ) -> Self {
        Self {
            visible_macd,
            min_macd,
            max_macd,
        }
    }

    /// Vérifie si le snapshot contient des données valides
    pub fn is_valid(&self) -> bool {
        !self.visible_macd.is_empty() 
            && self.min_macd.is_finite() 
            && self.max_macd.is_finite()
    }
}

