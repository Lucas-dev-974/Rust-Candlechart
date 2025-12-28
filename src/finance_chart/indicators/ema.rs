//! Module EMA (Exponential Moving Average) réutilisable
//!
//! Fournit une implémentation stateful pour le calcul de l'EMA.
//! Utilisé par les indicateurs MACD, RSI et autres.

/// Calculateur EMA stateful
/// 
/// Maintient l'état interne pour des calculs incrémentaux efficaces.
#[derive(Debug, Clone)]
pub struct Ema {
    /// Période de l'EMA
    period: usize,
    /// Multiplicateur alpha = 2 / (period + 1)
    alpha: f64,
    /// Valeur EMA précédente
    prev: Option<f64>,
    /// Historique des valeurs pour le seed SMA
    values: Vec<f64>,
}

impl Ema {
    /// Crée un nouveau calculateur EMA
    pub fn new(period: usize) -> Self {
        Self {
            period,
            alpha: 2.0 / (period + 1) as f64,
            prev: None,
            values: Vec::with_capacity(period),
        }
    }

    /// Alimente le calculateur avec une nouvelle valeur et retourne l'EMA résultante
    /// 
    /// # Arguments
    /// * `value` - Nouvelle valeur à intégrer
    /// 
    /// # Returns
    /// La valeur EMA calculée (peut être moins précise avant `period` valeurs)
    pub fn feed(&mut self, value: f64) -> f64 {
        self.values.push(value);
        let i = self.values.len() - 1;

        let result = if i == 0 {
            value
        } else if i == self.period - 1 {
            // Seed avec SMA
            self.values[0..self.period].iter().sum::<f64>() / self.period as f64
        } else if i < self.period - 1 {
            // Pas encore initialisé, retourner la valeur brute
            value
        } else {
            // Calcul EMA normal
            match self.prev {
                Some(prev) => (value - prev) * self.alpha + prev,
                None => value,
            }
        };

        self.prev = Some(result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_stateful() {
        let mut ema = Ema::new(3);
        
        let v1 = ema.feed(10.0);
        assert_eq!(v1, 10.0);
        
        let v2 = ema.feed(20.0);
        assert_eq!(v2, 20.0);
        
        // À period-1 (index 2), seed avec SMA
        let v3 = ema.feed(30.0);
        assert_eq!(v3, 20.0); // SMA de [10, 20, 30]
        
        // Après, calcul EMA normal
        let alpha = 2.0 / 4.0; // period = 3
        let v4 = ema.feed(40.0);
        let expected = (40.0 - 20.0) * alpha + 20.0;
        assert!((v4 - expected).abs() < 1e-10);
    }
}

