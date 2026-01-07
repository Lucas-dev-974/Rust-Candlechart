//! Persistance de la liste des actifs disponibles
//!
//! Ce module gère la sauvegarde et le chargement de la liste des actifs
//! depuis le provider dans un fichier JSON pour éviter de recharger à chaque fois.

use serde::{Deserialize, Serialize};
use crate::finance_chart::providers::binance::BinanceSymbol;

/// État des actifs à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsPersistenceState {
    /// Liste des actifs disponibles
    pub assets: Vec<BinanceSymbol>,
    /// Date de dernière mise à jour (timestamp Unix en secondes)
    #[serde(default)]
    pub last_updated: Option<u64>,
}

impl AssetsPersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let json = std::fs::read_to_string(path)?;
        let state: AssetsPersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Met à jour la liste des actifs en fusionnant avec les nouveaux
    /// Retourne le nombre de nouveaux actifs ajoutés
    pub fn update_assets(&mut self, new_assets: Vec<BinanceSymbol>) -> usize {
        let existing_symbols: std::collections::HashSet<String> = self.assets
            .iter()
            .map(|a| a.symbol.clone())
            .collect();
        
        let mut new_count = 0;
        for asset in new_assets {
            if !existing_symbols.contains(&asset.symbol) {
                self.assets.push(asset);
                new_count += 1;
            }
        }
        
        // Trier par volume 24h décroissant (popularité)
        self.assets.sort_by(|a, b| {
            let vol_a = a.volume_24h.unwrap_or(0.0);
            let vol_b = b.volume_24h.unwrap_or(0.0);
            vol_b.partial_cmp(&vol_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Mettre à jour le timestamp
        self.last_updated = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        
        new_count
    }
    
    /// Remplace complètement la liste des actifs
    pub fn replace_assets(&mut self, assets: Vec<BinanceSymbol>) {
        self.assets = assets;
        // Trier par volume 24h décroissant (popularité)
        self.assets.sort_by(|a, b| {
            let vol_a = a.volume_24h.unwrap_or(0.0);
            let vol_b = b.volume_24h.unwrap_or(0.0);
            vol_b.partial_cmp(&vol_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        // Mettre à jour le timestamp
        self.last_updated = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }
}

impl Default for AssetsPersistenceState {
    fn default() -> Self {
        Self {
            assets: Vec::new(),
            last_updated: None,
        }
    }
}

