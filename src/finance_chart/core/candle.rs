/// Représente une bougie OHLC (Open, High, Low, Close)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Timestamp Unix (secondes)
    pub timestamp: i64,
    /// Prix d'ouverture
    pub open: f64,
    /// Prix le plus haut
    pub high: f64,
    /// Prix le plus bas
    pub low: f64,
    /// Prix de clôture
    pub close: f64,
}

impl Candle {
    /// Crée une nouvelle bougie avec validation automatique
    /// 
    /// Les valeurs high et low sont automatiquement ajustées pour respecter
    /// les invariants OHLC : low ≤ min(open, close) et high ≥ max(open, close)
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64) -> Self {
        // Assurer que high >= max(open, close) et low <= min(open, close)
        let actual_high = high.max(open).max(close);
        let actual_low = low.min(open).min(close);
        
        Self {
            timestamp,
            open,
            high: actual_high,
            low: actual_low,
            close,
        }
    }

    /// Retourne true si la bougie est haussière (close > open)
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bullish_candle() {
        let candle = Candle::new(1000, 100.0, 105.0, 99.0, 104.0);
        assert!(candle.is_bullish());
        assert_eq!(candle.high, 105.0);
        assert_eq!(candle.low, 99.0);
    }

    #[test]
    fn test_bearish_candle() {
        let candle = Candle::new(1000, 104.0, 105.0, 99.0, 100.0);
        assert!(!candle.is_bullish());
    }
}

