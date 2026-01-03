//! État de trading pour les ordres d'achat/vente

use crate::app::data::{TradeHistory, OrderType};

/// État de trading pour gérer les ordres
#[derive(Debug, Clone)]
pub struct TradingState {
    /// Quantité à trader (en string pour l'input)
    pub order_quantity: String,
    /// Type d'ordre (Market ou Limit)
    pub order_type: OrderType,
    /// Prix limite (pour les ordres limit)
    pub limit_price: String,
    /// Take Profit (en string pour l'input)
    pub take_profit: String,
    /// Stop Loss (en string pour l'input)
    pub stop_loss: String,
    /// TP/SL activé (pour les ordres Market)
    pub tp_sl_enabled: bool,
    /// Historique des trades et positions
    pub trade_history: TradeHistory,
}

impl Default for TradingState {
    fn default() -> Self {
        Self {
            order_quantity: String::from("0.001"),
            order_type: OrderType::Market,
            limit_price: String::new(),
            take_profit: String::new(),
            stop_loss: String::new(),
            tp_sl_enabled: true,  // Activé par défaut
            trade_history: TradeHistory::new(),
        }
    }
}

impl TradingState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Parse la quantité en f64, retourne None si invalide
    pub fn parse_quantity(&self) -> Option<f64> {
        self.order_quantity.parse::<f64>().ok()
    }
    
    /// Parse le prix limite en f64, retourne None si invalide
    pub fn parse_limit_price(&self) -> Option<f64> {
        self.limit_price.parse::<f64>().ok()
    }
    
    /// Parse le Take Profit en f64, retourne None si invalide ou vide
    pub fn parse_take_profit(&self) -> Option<f64> {
        if self.take_profit.is_empty() {
            None
        } else {
            self.take_profit.parse::<f64>().ok()
        }
    }
    
    /// Parse le Stop Loss en f64, retourne None si invalide ou vide
    pub fn parse_stop_loss(&self) -> Option<f64> {
        if self.stop_loss.is_empty() {
            None
        } else {
            self.stop_loss.parse::<f64>().ok()
        }
    }
    
    /// Met à jour TP et SL avec 15% d'écart par rapport au prix actuel
    /// Ne met à jour que si les champs sont vides (pour ne pas écraser les valeurs saisies manuellement)
    pub fn update_tp_sl_from_price(&mut self, current_price: f64) {
        // Calculer TP (+15%) et SL (-15%)
        let take_profit_value = current_price * 1.15;
        let stop_loss_value = current_price * 0.85;
        
        // Ne mettre à jour que si les champs sont vides
        if self.take_profit.is_empty() {
            self.take_profit = format!("{:.2}", take_profit_value);
        }
        
        if self.stop_loss.is_empty() {
            self.stop_loss = format!("{:.2}", stop_loss_value);
        }
    }
}

