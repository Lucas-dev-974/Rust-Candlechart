//! Persistance des stratégies de trading
//!
//! Ce module gère la sauvegarde et le chargement des stratégies avec toute leur configuration
//! dans un fichier JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::app::strategies::manager::{StrategyStatus, RegisteredStrategy};
use crate::app::strategies::strategy::{TradingStrategy, TradingMode};

/// Type de stratégie (pour la désérialisation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    RSI,
    MovingAverageCrossover,
}

/// État sérialisable d'une stratégie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPersistenceState {
    /// ID unique de la stratégie
    pub id: String,
    /// Type de stratégie
    pub strategy_type: StrategyType,
    /// Nom de la stratégie
    pub name: String,
    /// Paramètres de la stratégie (nom -> valeur)
    pub parameters: HashMap<String, f64>,
    /// État de la stratégie
    pub status: StrategyStatus,
    /// Indique si la stratégie est activée
    pub enabled: bool,
    /// Timeframes autorisés (None = tous les timeframes)
    pub allowed_timeframes: Option<Vec<String>>,
    /// Mode de trading (achats uniquement, ventes uniquement, ou les deux)
    #[serde(default)]
    pub trading_mode: TradingMode,
}

/// État complet de toutes les stratégies à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesPersistenceState {
    /// Liste des stratégies
    pub strategies: Vec<StrategyPersistenceState>,
    /// Prochain ID à utiliser
    #[serde(default)]
    pub next_id: u64,
}

impl StrategiesPersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let json = std::fs::read_to_string(path)?;
        let state: StrategiesPersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for StrategiesPersistenceState {
    fn default() -> Self {
        Self {
            strategies: Vec::new(),
            next_id: 1,
        }
    }
}

/// Convertit une RegisteredStrategy en StrategyPersistenceState
pub fn strategy_to_persistence(
    id: String,
    reg: &RegisteredStrategy,
) -> Result<StrategyPersistenceState, String> {
    // Déterminer le type de stratégie
    let strategy_type = if reg.strategy.name().contains("RSI") {
        StrategyType::RSI
    } else if reg.strategy.name().contains("MA Crossover") || reg.strategy.name().contains("Moving Average") {
        StrategyType::MovingAverageCrossover
    } else {
        return Err(format!("Type de stratégie inconnu: {}", reg.strategy.name()));
    };
    
    // Extraire les paramètres
    let mut parameters = HashMap::new();
    for param in reg.strategy.parameters() {
        parameters.insert(param.name, param.value);
    }
    
    Ok(StrategyPersistenceState {
        id,
        strategy_type,
        name: reg.strategy.name().to_string(),
        parameters,
        status: reg.status.clone(),
        enabled: reg.enabled,
        allowed_timeframes: reg.allowed_timeframes.clone(),
        trading_mode: reg.trading_mode,
    })
}

/// Reconstruit une RegisteredStrategy depuis un StrategyPersistenceState
pub fn persistence_to_strategy(
    state: &StrategyPersistenceState,
) -> Result<RegisteredStrategy, String> {
    use crate::app::strategies::examples::{RSIStrategy, MovingAverageCrossoverStrategy};
    
    // Créer la stratégie selon son type
    let strategy: Box<dyn TradingStrategy> = match state.strategy_type {
        StrategyType::RSI => {
            let mut s = RSIStrategy::new();
            // Appliquer les paramètres sauvegardés
            for (name, value) in &state.parameters {
                if let Err(e) = s.update_parameter(name, *value) {
                    eprintln!("⚠️ Erreur lors de la restauration du paramètre {}: {}", name, e);
                }
            }
            Box::new(s)
        }
        StrategyType::MovingAverageCrossover => {
            let mut s = MovingAverageCrossoverStrategy::new();
            // Appliquer les paramètres sauvegardés
            for (name, value) in &state.parameters {
                if let Err(e) = s.update_parameter(name, *value) {
                    eprintln!("⚠️ Erreur lors de la restauration du paramètre {}: {}", name, e);
                }
            }
            Box::new(s)
        }
    };
    
    // Mettre à jour le nom si nécessaire
    if strategy.name() != state.name {
        // Note: On ne peut pas changer le nom directement, mais on peut le vérifier
        eprintln!("⚠️ Nom de stratégie différent: attendu '{}', obtenu '{}'", state.name, strategy.name());
    }
    
    Ok(RegisteredStrategy {
        strategy,
        status: state.status.clone(),
        enabled: state.enabled,
        allowed_timeframes: state.allowed_timeframes.clone(),
        trading_mode: state.trading_mode,
    })
}

