//! Exemples de stratégies de trading

use crate::app::strategies::strategy::*;
use crate::app::data::OrderType;
use crate::finance_chart::core::Candle;

/// Stratégie basée sur RSI (Relative Strength Index)
pub struct RSIStrategy {
    name: String,
    rsi_period: f64,
    oversold_threshold: f64,  // RSI < 30 = survente = signal d'achat
    overbought_threshold: f64, // RSI > 70 = surachat = signal de vente
    quantity: f64,
}

impl RSIStrategy {
    pub fn new() -> Self {
        Self {
            name: "RSI Strategy".to_string(),
            rsi_period: 14.0,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            quantity: 0.001,
        }
    }
    
    fn calculate_rsi(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period + 1 {
            return None;
        }
        
        let mut gains = 0.0;
        let mut losses = 0.0;
        
        for i in (candles.len() - period)..candles.len() {
            if i > 0 {
                let change = candles[i].close - candles[i - 1].close;
                if change > 0.0 {
                    gains += change;
                } else {
                    losses += change.abs();
                }
            }
        }
        
        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;
        
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        
        Some(rsi)
    }
}

impl TradingStrategy for RSIStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Stratégie basée sur l'indicateur RSI. Achete quand RSI < 30, vends quand RSI > 70"
    }
    
    fn evaluate(&self, context: &MarketContext) -> StrategyResult {
        let period = self.rsi_period as usize;
        
        if let Some(rsi) = self.calculate_rsi(&context.candles, period) {
            if rsi < self.oversold_threshold {
                StrategyResult {
                    signal: TradingSignal::Buy {
                        quantity: self.quantity,
                        order_type: OrderType::Market,
                        limit_price: None,
                        take_profit: Some(context.current_price * 1.05), // +5%
                        stop_loss: Some(context.current_price * 0.95),   // -5%
                    },
                    reason: format!("RSI ({:.2}) en survente (< {})", rsi, self.oversold_threshold),
                    confidence: ((self.oversold_threshold - rsi) / self.oversold_threshold).min(1.0),
                }
            } else if rsi > self.overbought_threshold {
                StrategyResult {
                    signal: TradingSignal::Sell {
                        quantity: self.quantity,
                        order_type: OrderType::Market,
                        limit_price: None,
                        take_profit: Some(context.current_price * 0.95), // -5%
                        stop_loss: Some(context.current_price * 1.05),   // +5%
                    },
                    reason: format!("RSI ({:.2}) en surachat (> {})", rsi, self.overbought_threshold),
                    confidence: ((rsi - self.overbought_threshold) / (100.0 - self.overbought_threshold)).min(1.0),
                }
            } else {
                StrategyResult {
                    signal: TradingSignal::Hold,
                    reason: format!("RSI ({:.2}) dans la zone neutre", rsi),
                    confidence: 0.0,
                }
            }
        } else {
            StrategyResult {
                signal: TradingSignal::Hold,
                reason: "Pas assez de données pour calculer le RSI".to_string(),
                confidence: 0.0,
            }
        }
    }
    
    fn parameters(&self) -> Vec<StrategyParameter> {
        vec![
            StrategyParameter {
                name: "rsi_period".to_string(),
                value: self.rsi_period,
                min: 5.0,
                max: 50.0,
                description: "Période pour le calcul du RSI".to_string(),
            },
            StrategyParameter {
                name: "oversold_threshold".to_string(),
                value: self.oversold_threshold,
                min: 10.0,
                max: 40.0,
                description: "Seuil de survente (signal d'achat)".to_string(),
            },
            StrategyParameter {
                name: "overbought_threshold".to_string(),
                value: self.overbought_threshold,
                min: 60.0,
                max: 90.0,
                description: "Seuil de surachat (signal de vente)".to_string(),
            },
            StrategyParameter {
                name: "quantity".to_string(),
                value: self.quantity,
                min: 0.0001,
                max: 1.0,
                description: "Quantité à trader".to_string(),
            },
        ]
    }
    
    fn update_parameter(&mut self, name: &str, value: f64) -> Result<(), String> {
        match name {
            "rsi_period" => {
                if value >= 5.0 && value <= 50.0 {
                    self.rsi_period = value;
                    Ok(())
                } else {
                    Err("Période RSI doit être entre 5 et 50".to_string())
                }
            }
            "oversold_threshold" => {
                if value >= 10.0 && value <= 40.0 {
                    self.oversold_threshold = value;
                    Ok(())
                } else {
                    Err("Seuil de survente doit être entre 10 et 40".to_string())
                }
            }
            "overbought_threshold" => {
                if value >= 60.0 && value <= 90.0 {
                    self.overbought_threshold = value;
                    Ok(())
                } else {
                    Err("Seuil de surachat doit être entre 60 et 90".to_string())
                }
            }
            "quantity" => {
                if value > 0.0 && value <= 1.0 {
                    self.quantity = value;
                    Ok(())
                } else {
                    Err("Quantité doit être positive et <= 1.0".to_string())
                }
            }
            _ => Err(format!("Paramètre inconnu: {}", name)),
        }
    }
    
    fn clone_box(&self) -> Box<dyn TradingStrategy> {
        Box::new(Self {
            name: self.name.clone(),
            rsi_period: self.rsi_period,
            oversold_threshold: self.oversold_threshold,
            overbought_threshold: self.overbought_threshold,
            quantity: self.quantity,
        })
    }
}

/// Stratégie basée sur les moyennes mobiles (crossover)
pub struct MovingAverageCrossoverStrategy {
    name: String,
    fast_period: f64,
    slow_period: f64,
    quantity: f64,
}

impl MovingAverageCrossoverStrategy {
    pub fn new() -> Self {
        Self {
            name: "MA Crossover Strategy".to_string(),
            fast_period: 10.0,
            slow_period: 30.0,
            quantity: 0.001,
        }
    }
    
    fn calculate_ma(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period {
            return None;
        }
        
        let sum: f64 = candles[candles.len() - period..]
            .iter()
            .map(|c| c.close)
            .sum();
        
        Some(sum / period as f64)
    }
}

impl TradingStrategy for MovingAverageCrossoverStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Stratégie de crossover de moyennes mobiles. Achete quand MA rapide croise au-dessus de MA lente"
    }
    
    fn evaluate(&self, context: &MarketContext) -> StrategyResult {
        let fast_period = self.fast_period as usize;
        let slow_period = self.slow_period as usize;
        
        if let (Some(fast_ma), Some(slow_ma)) = (
            self.calculate_ma(&context.candles, fast_period),
            self.calculate_ma(&context.candles, slow_period),
        ) {
            // Calculer les MA précédentes pour détecter le crossover
            if context.candles.len() >= slow_period + 1 {
                let prev_fast_ma = self.calculate_ma(
                    &context.candles[..context.candles.len() - 1],
                    fast_period
                ).unwrap_or(fast_ma);
                let prev_slow_ma = self.calculate_ma(
                    &context.candles[..context.candles.len() - 1],
                    slow_period
                ).unwrap_or(slow_ma);
                
                // Crossover haussier: fast_ma croise au-dessus de slow_ma
                if prev_fast_ma <= prev_slow_ma && fast_ma > slow_ma {
                    StrategyResult {
                        signal: TradingSignal::Buy {
                            quantity: self.quantity,
                            order_type: OrderType::Market,
                            limit_price: None,
                            take_profit: Some(context.current_price * 1.10),
                            stop_loss: Some(context.current_price * 0.95),
                        },
                        reason: format!("Crossover haussier: MA{} ({:.2}) > MA{} ({:.2})", 
                            fast_period, fast_ma, slow_period, slow_ma),
                        confidence: 0.7,
                    }
                }
                // Crossover baissier: fast_ma croise en-dessous de slow_ma
                else if prev_fast_ma >= prev_slow_ma && fast_ma < slow_ma {
                    StrategyResult {
                        signal: TradingSignal::Sell {
                            quantity: self.quantity,
                            order_type: OrderType::Market,
                            limit_price: None,
                            take_profit: Some(context.current_price * 0.90),
                            stop_loss: Some(context.current_price * 1.05),
                        },
                        reason: format!("Crossover baissier: MA{} ({:.2}) < MA{} ({:.2})", 
                            fast_period, fast_ma, slow_period, slow_ma),
                        confidence: 0.7,
                    }
                } else {
                    StrategyResult {
                        signal: TradingSignal::Hold,
                        reason: format!("Pas de crossover: MA{}={:.2}, MA{}={:.2}", 
                            fast_period, fast_ma, slow_period, slow_ma),
                        confidence: 0.0,
                    }
                }
            } else {
                StrategyResult {
                    signal: TradingSignal::Hold,
                    reason: "Pas assez de données pour détecter un crossover".to_string(),
                    confidence: 0.0,
                }
            }
        } else {
            StrategyResult {
                signal: TradingSignal::Hold,
                reason: "Pas assez de données pour calculer les moyennes mobiles".to_string(),
                confidence: 0.0,
            }
        }
    }
    
    fn parameters(&self) -> Vec<StrategyParameter> {
        vec![
            StrategyParameter {
                name: "fast_period".to_string(),
                value: self.fast_period,
                min: 5.0,
                max: 50.0,
                description: "Période de la moyenne mobile rapide".to_string(),
            },
            StrategyParameter {
                name: "slow_period".to_string(),
                value: self.slow_period,
                min: 10.0,
                max: 200.0,
                description: "Période de la moyenne mobile lente".to_string(),
            },
            StrategyParameter {
                name: "quantity".to_string(),
                value: self.quantity,
                min: 0.0001,
                max: 1.0,
                description: "Quantité à trader".to_string(),
            },
        ]
    }
    
    fn update_parameter(&mut self, name: &str, value: f64) -> Result<(), String> {
        match name {
            "fast_period" => {
                if value >= 5.0 && value <= 50.0 && value < self.slow_period {
                    self.fast_period = value;
                    Ok(())
                } else {
                    Err("Période rapide doit être entre 5 et 50, et < période lente".to_string())
                }
            }
            "slow_period" => {
                if value >= 10.0 && value <= 200.0 && value > self.fast_period {
                    self.slow_period = value;
                    Ok(())
                } else {
                    Err("Période lente doit être entre 10 et 200, et > période rapide".to_string())
                }
            }
            "quantity" => {
                if value > 0.0 && value <= 1.0 {
                    self.quantity = value;
                    Ok(())
                } else {
                    Err("Quantité doit être positive et <= 1.0".to_string())
                }
            }
            _ => Err(format!("Paramètre inconnu: {}", name)),
        }
    }
    
    fn clone_box(&self) -> Box<dyn TradingStrategy> {
        Box::new(Self {
            name: self.name.clone(),
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            quantity: self.quantity,
        })
    }
}


