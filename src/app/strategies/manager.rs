//! Gestionnaire de stratégies de trading

use std::collections::HashMap;
use crate::app::strategies::strategy::{TradingStrategy, MarketContext, StrategyResult, TradingMode};
use serde::{Serialize, Deserialize};

/// État d'une stratégie
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyStatus {
    /// Stratégie active et en cours d'exécution
    Active,
    /// Stratégie désactivée
    Inactive,
    /// Stratégie en pause
    Paused,
}

/// Stratégie enregistrée avec son état
#[derive(Clone)]
pub struct RegisteredStrategy {
    pub strategy: Box<dyn TradingStrategy>,
    pub status: StrategyStatus,
    pub enabled: bool,
    /// Timeframes autorisés pour cette stratégie (None = tous les timeframes)
    pub allowed_timeframes: Option<Vec<String>>,
    /// Mode de trading (achats uniquement, ventes uniquement, ou les deux)
    pub trading_mode: TradingMode,
}

// Implémentation manuelle de Debug pour RegisteredStrategy
impl std::fmt::Debug for RegisteredStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredStrategy")
            .field("name", &self.strategy.name())
            .field("status", &self.status)
            .field("enabled", &self.enabled)
            .field("allowed_timeframes", &self.allowed_timeframes)
            .field("trading_mode", &self.trading_mode)
            .finish()
    }
}

/// Gestionnaire de toutes les stratégies
#[derive(Debug, Clone)]
pub struct StrategyManager {
    /// Stratégies enregistrées (ID -> Stratégie)
    strategies: HashMap<String, RegisteredStrategy>,
    /// Compteur pour générer des IDs uniques
    next_id: u64,
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Enregistre une nouvelle stratégie
    pub fn register_strategy(&mut self, strategy: Box<dyn TradingStrategy>) -> String {
        self.register_strategy_with_timeframes(strategy, None)
    }
    
    /// Enregistre une nouvelle stratégie avec des timeframes spécifiques
    pub fn register_strategy_with_timeframes(
        &mut self,
        strategy: Box<dyn TradingStrategy>,
        allowed_timeframes: Option<Vec<String>>,
    ) -> String {
        let id = format!("strategy_{}", self.next_id);
        self.next_id += 1;
        
        self.strategies.insert(id.clone(), RegisteredStrategy {
            strategy,
            status: StrategyStatus::Inactive,
            enabled: false,
            allowed_timeframes,
            trading_mode: TradingMode::Both,
        });
        
        id
    }
    
    /// Active une stratégie
    pub fn enable_strategy(&mut self, id: &str) -> Result<(), String> {
        if let Some(reg) = self.strategies.get_mut(id) {
            reg.enabled = true;
            reg.status = StrategyStatus::Active;
            Ok(())
        } else {
            Err(format!("Stratégie {} introuvable", id))
        }
    }
    
    /// Désactive une stratégie
    pub fn disable_strategy(&mut self, id: &str) -> Result<(), String> {
        if let Some(reg) = self.strategies.get_mut(id) {
            reg.enabled = false;
            reg.status = StrategyStatus::Inactive;
            Ok(())
        } else {
            Err(format!("Stratégie {} introuvable", id))
        }
    }
    
    /// Évalue toutes les stratégies actives pour un timeframe donné
    pub fn evaluate_all(&self, context: &MarketContext, current_interval: &str) -> Vec<(String, StrategyResult)> {
        self.strategies
            .iter()
            .filter(|(_, reg)| {
                // Vérifier que la stratégie est activée
                if !reg.enabled || reg.status != StrategyStatus::Active {
                    return false;
                }
                
                // Vérifier le timeframe si spécifié
                if let Some(ref allowed) = reg.allowed_timeframes {
                    allowed.contains(&current_interval.to_string())
                } else {
                    // Aucune restriction de timeframe
                    true
                }
            })
            .map(|(id, reg)| {
                let result = reg.strategy.evaluate(context);
                (id.clone(), result)
            })
            .collect()
    }
    
    /// Met à jour les timeframes autorisés pour une stratégie
    pub fn set_strategy_timeframes(&mut self, id: &str, timeframes: Option<Vec<String>>) -> Result<(), String> {
        if let Some(reg) = self.strategies.get_mut(id) {
            reg.allowed_timeframes = timeframes;
            Ok(())
        } else {
            Err(format!("Stratégie {} introuvable", id))
        }
    }
    
    /// Met à jour le mode de trading pour une stratégie
    pub fn set_strategy_trading_mode(&mut self, id: &str, trading_mode: TradingMode) -> Result<(), String> {
        if let Some(reg) = self.strategies.get_mut(id) {
            reg.trading_mode = trading_mode;
            Ok(())
        } else {
            Err(format!("Stratégie {} introuvable", id))
        }
    }
    
    /// Retourne toutes les stratégies
    pub fn get_all(&self) -> Vec<(String, &RegisteredStrategy)> {
        self.strategies
            .iter()
            .map(|(id, reg)| (id.clone(), reg))
            .collect()
    }
    
    /// Retourne une stratégie par ID
    pub fn get_strategy(&self, id: &str) -> Option<&RegisteredStrategy> {
        self.strategies.get(id)
    }
    
    /// Retourne une stratégie mutable par ID
    pub fn get_strategy_mut(&mut self, id: &str) -> Option<&mut RegisteredStrategy> {
        self.strategies.get_mut(id)
    }
    
    /// Supprime une stratégie
    pub fn remove_strategy(&mut self, id: &str) -> Result<(), String> {
        if self.strategies.remove(id).is_some() {
            Ok(())
        } else {
            Err(format!("Stratégie {} introuvable", id))
        }
    }
    
    /// Sauvegarde toutes les stratégies dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::app::persistence::{StrategiesPersistenceState, strategy_to_persistence};
        
        let mut persistence_state = StrategiesPersistenceState {
            strategies: Vec::new(),
            next_id: self.next_id,
        };
        
        // Convertir toutes les stratégies
        for (id, reg) in &self.strategies {
            match strategy_to_persistence(id.clone(), reg) {
                Ok(state) => persistence_state.strategies.push(state),
                Err(e) => eprintln!("⚠️ Erreur lors de la sauvegarde de la stratégie {}: {}", id, e),
            }
        }
        
        persistence_state.save_to_file(path)
    }
    
    /// Charge les stratégies depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::app::persistence::{StrategiesPersistenceState, persistence_to_strategy};
        
        let persistence_state = StrategiesPersistenceState::load_from_file(path)?;
        
        let mut manager = Self {
            strategies: HashMap::new(),
            next_id: persistence_state.next_id,
        };
        
        // Reconstruire toutes les stratégies
        for state in persistence_state.strategies {
            match persistence_to_strategy(&state) {
                Ok(reg) => {
                    manager.strategies.insert(state.id, reg);
                }
                Err(e) => {
                    eprintln!("⚠️ Erreur lors du chargement de la stratégie {}: {}", state.id, e);
                }
            }
        }
        
        Ok(manager)
    }
    
    /// Crée un nouveau StrategyManager, en chargeant depuis un fichier si disponible
    pub fn new_or_load(path: &str) -> Self {
        Self::load_from_file(path).unwrap_or_else(|e| {
            eprintln!("⚠️ Impossible de charger les stratégies depuis {}: {}", path, e);
            eprintln!("   Création d'un nouveau gestionnaire de stratégies");
            Self::new()
        })
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self::new()
    }
}


