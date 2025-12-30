//! Persistance du timeframe sélectionné
//!
//! Ce module gère la sauvegarde et le chargement du timeframe (intervalle) sélectionné
//! dans un fichier JSON pour restaurer la sélection au prochain démarrage.

use serde::{Deserialize, Serialize};

/// État du timeframe à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframePersistenceState {
    /// Intervalle sélectionné (ex: "1h", "15m", "1d")
    pub interval: String,
    /// Symbole associé au timeframe (optionnel, pour prioriser la série avec ce symbole)
    #[serde(default)]
    pub symbol: Option<String>,
}

impl TimeframePersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let state: TimeframePersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for TimeframePersistenceState {
    fn default() -> Self {
        Self {
            interval: String::from("1h"), // Valeur par défaut
            symbol: None,
        }
    }
}

