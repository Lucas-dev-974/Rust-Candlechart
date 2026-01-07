//! Persistance des actifs sélectionnés
//!
//! Ce module gère la sauvegarde et le chargement des actifs sélectionnés
//! dans un fichier JSON pour restaurer la sélection au prochain démarrage.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// État des actifs sélectionnés à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedAssetsPersistenceState {
    /// Liste des symboles d'actifs sélectionnés
    pub selected_symbols: Vec<String>,
}

impl SelectedAssetsPersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let json = std::fs::read_to_string(path)?;
        let state: SelectedAssetsPersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Convertit en HashSet
    pub fn to_hashset(&self) -> HashSet<String> {
        self.selected_symbols.iter().cloned().collect()
    }
    
    /// Crée depuis un HashSet
    pub fn from_hashset(selected: &HashSet<String>) -> Self {
        let mut symbols: Vec<String> = selected.iter().cloned().collect();
        symbols.sort(); // Trier pour un affichage cohérent
        Self {
            selected_symbols: symbols,
        }
    }
}

impl Default for SelectedAssetsPersistenceState {
    fn default() -> Self {
        Self {
            selected_symbols: Vec::new(),
        }
    }
}

