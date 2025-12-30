//! Persistance des données de trading (paper trading)
//!
//! Ce module gère la sauvegarde et le chargement de l'historique des trades
//! et des positions ouvertes dans un fichier JSON.

use serde::{Deserialize, Serialize};
use crate::app::data::{TradeHistory, Trade, Position, PendingOrder};

/// État complet du trading à sauvegarder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPersistenceState {
    /// Historique des trades
    pub trades: Vec<Trade>,
    /// Positions ouvertes
    pub open_positions: Vec<Position>,
    /// Ordres en attente (limit orders)
    #[serde(default)]
    pub pending_orders: Vec<PendingOrder>,
    /// Prochain ID de trade
    #[serde(default)]
    pub next_trade_id: u64,
    /// Prochain ID d'ordre
    #[serde(default)]
    pub next_order_id: u64,
}

impl TradingPersistenceState {
    /// Charge l'état depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let state: TradingPersistenceState = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    /// Sauvegarde l'état dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Convertit en TradeHistory
    pub fn to_trade_history(&self) -> TradeHistory {
        TradeHistory {
            open_positions: self.open_positions.clone(),
            trades: self.trades.clone(),
            pending_orders: self.pending_orders.clone(),
            next_trade_id: self.next_trade_id,
            next_order_id: self.next_order_id,
        }
    }
}

impl From<&TradeHistory> for TradingPersistenceState {
    fn from(trade_history: &TradeHistory) -> Self {
        Self {
            trades: trade_history.trades.clone(),
            open_positions: trade_history.open_positions.clone(),
            pending_orders: trade_history.pending_orders.clone(),
            next_trade_id: trade_history.next_trade_id,
            next_order_id: trade_history.next_order_id,
        }
    }
}

impl Default for TradingPersistenceState {
    fn default() -> Self {
        Self {
            trades: Vec::new(),
            open_positions: Vec::new(),
            pending_orders: Vec::new(),
            next_trade_id: 1,
            next_order_id: 1,
        }
    }
}

