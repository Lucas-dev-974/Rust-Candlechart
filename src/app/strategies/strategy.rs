//! Trait de base pour les stratégies de trading

use crate::finance_chart::core::{Candle, SeriesId};
use crate::app::data::{TradeType, OrderType};

/// Contexte de marché fourni à une stratégie
#[derive(Debug, Clone)]
pub struct MarketContext {
    /// Symbole tradé
    pub symbol: String,
    /// Série ID
    pub series_id: SeriesId,
    /// Bougie actuelle
    pub current_candle: Candle,
    /// Historique des bougies (dernières N bougies)
    pub candles: Vec<Candle>,
    /// Prix actuel
    pub current_price: f64,
    /// Volume actuel
    pub current_volume: f64,
}

/// Mode de trading pour une stratégie
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TradingMode {
    /// Permet uniquement les achats
    BuyOnly,
    /// Permet uniquement les ventes
    SellOnly,
    /// Permet les achats et les ventes
    Both,
}

impl Default for TradingMode {
    fn default() -> Self {
        TradingMode::Both
    }
}

/// Signal généré par une stratégie
#[derive(Debug, Clone)]
pub enum TradingSignal {
    /// Acheter (ouvrir long ou fermer short)
    Buy {
        quantity: f64,
        order_type: OrderType,
        limit_price: Option<f64>,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
    },
    /// Vendre (ouvrir short ou fermer long)
    Sell {
        quantity: f64,
        order_type: OrderType,
        limit_price: Option<f64>,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
    },
    /// Ne rien faire
    Hold,
}

/// Résultat de l'évaluation d'une stratégie
#[derive(Debug, Clone)]
pub struct StrategyResult {
    /// Signal généré
    pub signal: TradingSignal,
    /// Raison du signal (pour le debugging)
    pub reason: String,
    /// Confiance du signal (0.0 à 1.0)
    pub confidence: f64,
}

/// Paramètre d'une stratégie
#[derive(Debug, Clone)]
pub struct StrategyParameter {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub description: String,
}

/// Trait que toutes les stratégies doivent implémenter
pub trait TradingStrategy: Send + Sync {
    /// Nom de la stratégie
    fn name(&self) -> &str;
    
    /// Description de la stratégie
    fn description(&self) -> &str;
    
    /// Évalue le marché et génère un signal
    fn evaluate(&self, context: &MarketContext) -> StrategyResult;
    
    /// Retourne les paramètres configurables de la stratégie
    fn parameters(&self) -> Vec<StrategyParameter>;
    
    /// Met à jour un paramètre
    fn update_parameter(&mut self, name: &str, value: f64) -> Result<(), String>;
    
    /// Clone la stratégie (pour permettre le stockage dans un Vec)
    fn clone_box(&self) -> Box<dyn TradingStrategy>;
}

// Implémentation de Clone pour Box<dyn TradingStrategy>
impl Clone for Box<dyn TradingStrategy> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


