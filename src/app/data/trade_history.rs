//! Historique des trades et gestion des positions

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Type de trade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
}

/// Type d'ordre
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Ordre au marché (exécution immédiate)
    Market,
    /// Ordre limit (exécution au prix spécifié ou meilleur)
    Limit,
}

/// Ordre en attente (limit order)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOrder {
    /// ID unique de l'ordre
    pub id: u64,
    /// Symbole tradé
    pub symbol: String,
    /// Type de trade
    pub trade_type: TradeType,
    /// Quantité
    pub quantity: f64,
    /// Prix limite
    pub limit_price: f64,
    /// Take Profit (optionnel)
    pub take_profit: Option<f64>,
    /// Stop Loss (optionnel)
    pub stop_loss: Option<f64>,
    /// Timestamp de création
    pub created_timestamp: i64,
}

/// Position ouverte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbole tradé (ex: "BTCUSDT")
    pub symbol: String,
    /// Quantité de la position
    pub quantity: f64,
    /// Prix d'entrée
    pub entry_price: f64,
    /// Timestamp d'ouverture
    pub open_timestamp: i64,
    /// Type de position (Buy = long, Sell = short)
    pub trade_type: TradeType,
    /// Take Profit (optionnel)
    pub take_profit: Option<f64>,
    /// Stop Loss (optionnel)
    pub stop_loss: Option<f64>,
}

impl Position {
    /// Calcule le P&L non réalisé de la position avec le prix actuel
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        match self.trade_type {
            TradeType::Buy => (current_price - self.entry_price) * self.quantity,
            TradeType::Sell => (self.entry_price - current_price) * self.quantity,
        }
    }
    
    /// Calcule la valeur de la marge utilisée
    pub fn margin_used(&self) -> f64 {
        self.entry_price * self.quantity
    }
}

/// Trade exécuté (historique)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// ID unique du trade
    pub id: u64,
    /// Symbole tradé
    pub symbol: String,
    /// Type de trade
    pub trade_type: TradeType,
    /// Quantité
    pub quantity: f64,
    /// Prix d'exécution
    pub price: f64,
    /// Montant total (quantity * price)
    pub total_amount: f64,
    /// P&L réalisé (si fermeture de position)
    pub realized_pnl: f64,
    /// Timestamp d'exécution
    pub timestamp: i64,
    /// ID de la stratégie qui a généré ce trade (None si trade manuel)
    #[serde(default)]
    pub strategy_id: Option<String>,
    /// Nom de la stratégie (pour affichage)
    #[serde(default)]
    pub strategy_name: Option<String>,
}

/// Gestionnaire de l'historique des trades et des positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    /// Positions ouvertes (par symbole)
    pub open_positions: Vec<Position>,
    /// Historique de tous les trades
    pub trades: Vec<Trade>,
    /// Ordres en attente (limit orders)
    pub pending_orders: Vec<PendingOrder>,
    /// Compteur pour générer des IDs uniques
    pub next_trade_id: u64,
    /// Compteur pour générer des IDs d'ordres uniques
    pub next_order_id: u64,
}

impl Default for TradeHistory {
    fn default() -> Self {
        Self {
            open_positions: Vec::new(),
            trades: Vec::new(),
            pending_orders: Vec::new(),
            next_trade_id: 1,
            next_order_id: 1,
        }
    }
}

impl TradeHistory {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Charge l'historique depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::app::persistence::TradingPersistenceState;
        let state = TradingPersistenceState::load_from_file(path)?;
        Ok(state.to_trade_history())
    }
    
    /// Sauvegarde l'historique dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::app::persistence::TradingPersistenceState;
        let state = TradingPersistenceState::from(self);
        state.save_to_file(path)
    }
    
    /// Ouvre une position (achat) avec TP et SL
    pub fn open_buy_position_with_tp_sl(
        &mut self,
        symbol: String,
        quantity: f64,
        price: f64,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
    ) -> Trade {
        self.open_buy_position_with_tp_sl_and_strategy(
            symbol, quantity, price, take_profit, stop_loss, None, None
        )
    }
    
    /// Ouvre une position (achat) avec TP, SL et informations de stratégie
    pub fn open_buy_position_with_tp_sl_and_strategy(
        &mut self,
        symbol: String,
        quantity: f64,
        price: f64,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
        strategy_id: Option<String>,
        strategy_name: Option<String>,
    ) -> Trade {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let position = Position {
            symbol: symbol.clone(),
            quantity,
            entry_price: price,
            open_timestamp: timestamp,
            trade_type: TradeType::Buy,
            take_profit,
            stop_loss,
        };
        
        self.open_positions.push(position);
        
        let trade = Trade {
            id: self.next_trade_id,
            symbol,
            trade_type: TradeType::Buy,
            quantity,
            price,
            total_amount: quantity * price,
            realized_pnl: 0.0,
            timestamp,
            strategy_id,
            strategy_name,
        };
        
        self.next_trade_id += 1;
        self.trades.push(trade.clone());
        
        trade
    }
    
    /// Ferme une position (vente)
    pub fn close_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Option<Trade> {
        self.close_position_with_strategy(symbol, quantity, price, None, None)
    }
    
    /// Ferme une position (vente) avec informations de stratégie
    pub fn close_position_with_strategy(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        strategy_id: Option<String>,
        strategy_name: Option<String>,
    ) -> Option<Trade> {
        // Trouver une position ouverte correspondante
        let position_index = self.open_positions.iter()
            .position(|p| p.symbol == symbol && p.trade_type == TradeType::Buy);
        
        if let Some(index) = position_index {
            let position = &self.open_positions[index];
            
            // Calculer le P&L réalisé
            let realized_pnl = (price - position.entry_price) * quantity.min(position.quantity);
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            
            let trade = Trade {
                id: self.next_trade_id,
                symbol: symbol.to_string(),
                trade_type: TradeType::Sell,
                quantity,
                price,
                total_amount: quantity * price,
                realized_pnl,
                timestamp,
                strategy_id,
                strategy_name,
            };
            
            self.next_trade_id += 1;
            self.trades.push(trade.clone());
            
            // Réduire ou supprimer la position
            if quantity >= position.quantity {
                self.open_positions.remove(index);
            } else {
                // Réduire la quantité de la position
                if let Some(pos) = self.open_positions.get_mut(index) {
                    pos.quantity -= quantity;
                }
            }
            
            Some(trade)
        } else {
            None
        }
    }
    
    /// Ouvre une position de vente (short) avec TP et SL
    pub fn open_sell_position_with_tp_sl(
        &mut self,
        symbol: String,
        quantity: f64,
        price: f64,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
    ) -> Trade {
        self.open_sell_position_with_tp_sl_and_strategy(
            symbol, quantity, price, take_profit, stop_loss, None, None
        )
    }
    
    /// Ouvre une position de vente (short) avec TP, SL et informations de stratégie
    pub fn open_sell_position_with_tp_sl_and_strategy(
        &mut self,
        symbol: String,
        quantity: f64,
        price: f64,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
        strategy_id: Option<String>,
        strategy_name: Option<String>,
    ) -> Trade {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let position = Position {
            symbol: symbol.clone(),
            quantity,
            entry_price: price,
            open_timestamp: timestamp,
            trade_type: TradeType::Sell,
            take_profit,
            stop_loss,
        };
        
        self.open_positions.push(position);
        
        let trade = Trade {
            id: self.next_trade_id,
            symbol,
            trade_type: TradeType::Sell,
            quantity,
            price,
            total_amount: quantity * price,
            realized_pnl: 0.0,
            timestamp,
            strategy_id,
            strategy_name,
        };
        
        self.next_trade_id += 1;
        self.trades.push(trade.clone());
        
        trade
    }
    
    /// Ferme une position short (achat)
    #[allow(dead_code)] // Pour usage futur
    pub fn close_short_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Option<Trade> {
        self.close_short_position_with_strategy(symbol, quantity, price, None, None)
    }
    
    /// Ferme une position short (achat) avec informations de stratégie
    pub fn close_short_position_with_strategy(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        strategy_id: Option<String>,
        strategy_name: Option<String>,
    ) -> Option<Trade> {
        // Trouver une position short ouverte correspondante
        let position_index = self.open_positions.iter()
            .position(|p| p.symbol == symbol && p.trade_type == TradeType::Sell);
        
        if let Some(index) = position_index {
            let position = &self.open_positions[index];
            
            // Calculer le P&L réalisé (pour un short: prix d'entrée - prix de sortie)
            let realized_pnl = (position.entry_price - price) * quantity.min(position.quantity);
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            
            let trade = Trade {
                id: self.next_trade_id,
                symbol: symbol.to_string(),
                trade_type: TradeType::Buy,
                quantity,
                price,
                total_amount: quantity * price,
                realized_pnl,
                timestamp,
                strategy_id,
                strategy_name,
            };
            
            self.next_trade_id += 1;
            self.trades.push(trade.clone());
            
            // Réduire ou supprimer la position
            if quantity >= position.quantity {
                self.open_positions.remove(index);
            } else {
                // Réduire la quantité de la position
                if let Some(pos) = self.open_positions.get_mut(index) {
                    pos.quantity -= quantity;
                }
            }
            
            Some(trade)
        } else {
            None
        }
    }
    
    /// Calcule le P&L non réalisé total avec le prix actuel d'un symbole
    pub fn total_unrealized_pnl(&self, symbol: &str, current_price: f64) -> f64 {
        self.open_positions
            .iter()
            .filter(|p| p.symbol == symbol)
            .map(|p| p.unrealized_pnl(current_price))
            .sum()
    }
    
    /// Calcule la marge totale utilisée pour un symbole
    pub fn total_margin_used(&self, symbol: &str) -> f64 {
        self.open_positions
            .iter()
            .filter(|p| p.symbol == symbol)
            .map(|p| p.margin_used())
            .sum()
    }
    
    /// Retourne le nombre de positions ouvertes
    pub fn open_positions_count(&self) -> u32 {
        self.open_positions.len() as u32
    }
    
    /// Retourne le P&L réalisé total
    pub fn total_realized_pnl(&self) -> f64 {
        self.trades.iter().map(|t| t.realized_pnl).sum()
    }
    
    /// Crée un ordre limit en attente
    pub fn create_pending_order(
        &mut self,
        symbol: String,
        trade_type: TradeType,
        quantity: f64,
        limit_price: f64,
        take_profit: Option<f64>,
        stop_loss: Option<f64>,
    ) -> PendingOrder {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let order = PendingOrder {
            id: self.next_order_id,
            symbol,
            trade_type,
            quantity,
            limit_price,
            take_profit,
            stop_loss,
            created_timestamp: timestamp,
        };
        
        self.next_order_id += 1;
        self.pending_orders.push(order.clone());
        
        order
    }
    
    /// Vérifie et exécute les ordres limit si le prix correspond
    pub fn check_and_execute_pending_orders(&mut self, symbol: &str, current_price: f64) {
        let mut orders_to_execute = Vec::new();
        
        // Trouver les ordres qui peuvent être exécutés
        for (index, order) in self.pending_orders.iter().enumerate() {
            if order.symbol != symbol {
                continue;
            }
            
            let should_execute = match order.trade_type {
                TradeType::Buy => current_price <= order.limit_price, // Achat: prix actuel <= prix limit
                TradeType::Sell => current_price >= order.limit_price, // Vente: prix actuel >= prix limit
            };
            
            if should_execute {
                orders_to_execute.push((index, order.clone()));
            }
        }
        
        // Exécuter les ordres (en ordre inverse pour éviter les problèmes d'index)
        for (index, order) in orders_to_execute.iter().rev() {
            // Retirer l'ordre de la liste
            self.pending_orders.remove(*index);
            
            // Exécuter l'ordre
            match order.trade_type {
                                TradeType::Buy => {
                                    // Ouvrir une position d'achat avec TP/SL
                                    self.open_buy_position_with_tp_sl(
                                        order.symbol.clone(),
                                        order.quantity,
                                        order.limit_price,
                                        order.take_profit,
                                        order.stop_loss,
                                    );
                                }
                                TradeType::Sell => {
                                    // Essayer de fermer une position existante
                                    if let Some(_trade) = self.close_position(&order.symbol, order.quantity, order.limit_price) {
                                        // Position fermée
                                    } else {
                                        // Ouvrir une position short avec TP/SL
                                        self.open_sell_position_with_tp_sl(
                                            order.symbol.clone(),
                                            order.quantity,
                                            order.limit_price,
                                            order.take_profit,
                                            order.stop_loss,
                                        );
                                    }
                                }
            }
        }
    }
    
    /// Vérifie et exécute les TP/SL des positions ouvertes
    pub fn check_take_profit_stop_loss(&mut self, symbol: &str, current_price: f64) {
        let mut positions_to_close = Vec::new();
        
        for (index, position) in self.open_positions.iter().enumerate() {
            if position.symbol != symbol {
                continue;
            }
            
            let should_close = match (position.take_profit, position.stop_loss, position.trade_type) {
                (Some(tp), Some(sl), TradeType::Buy) => {
                    current_price >= tp || current_price <= sl
                }
                (Some(tp), Some(sl), TradeType::Sell) => {
                    current_price <= tp || current_price >= sl
                }
                (Some(tp), None, TradeType::Buy) => current_price >= tp,
                (Some(tp), None, TradeType::Sell) => current_price <= tp,
                (None, Some(sl), TradeType::Buy) => current_price <= sl,
                (None, Some(sl), TradeType::Sell) => current_price >= sl,
                (None, None, _) => false,
            };
            
            if should_close {
                positions_to_close.push((index, position.clone()));
            }
        }
        
        // Fermer les positions (en ordre inverse)
        for (index, position) in positions_to_close.iter().rev() {
            match position.trade_type {
                TradeType::Buy => {
                    if let Some(trade) = self.close_position(&position.symbol, position.quantity, current_price) {
                        println!("  ✅ TP/SL déclenché: Position #{} fermée (P&L: {:.2} USDT)", 
                            position.symbol, trade.realized_pnl);
                    }
                }
                TradeType::Sell => {
                    if let Some(trade) = self.close_short_position(&position.symbol, position.quantity, current_price) {
                        println!("  ✅ TP/SL déclenché: Position short #{} fermée (P&L: {:.2} USDT)", 
                            position.symbol, trade.realized_pnl);
                    }
                }
            }
            self.open_positions.remove(*index);
        }
    }
}

