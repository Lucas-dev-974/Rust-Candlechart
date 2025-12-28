//! Indicateurs techniques pour le graphique financier
//!
//! Ce module contient les calculs et le rendu des indicateurs techniques
//! comme RSI, MACD, etc.

use crate::finance_chart::core::Candle;

/// Paramètres par défaut pour le RSI
pub const RSI_PERIOD: usize = 14;
pub const RSI_OVERBOUGHT: f64 = 70.0;
pub const RSI_OVERSOLD: f64 = 30.0;

/// Paramètres par défaut pour le MACD
pub const MACD_FAST_PERIOD: usize = 12;
pub const MACD_SLOW_PERIOD: usize = 26;
pub const MACD_SIGNAL_PERIOD: usize = 9;

/// Calcule le RSI (Relative Strength Index) pour une série de bougies
/// 
/// Le RSI est un indicateur de momentum qui mesure la vitesse et l'amplitude
/// des variations de prix. Il varie entre 0 et 100.
/// 
/// Cette implémentation utilise une moyenne mobile simple (SMA) pour calculer
/// les moyennes des gains et pertes.
/// 
/// # Arguments
/// * `candles` - Slice de bougies triées par timestamp croissant
/// * `period` - Période pour le calcul (défaut: 14)
/// 
/// # Retourne
/// Un vecteur de valeurs RSI correspondant à chaque bougie.
/// Les premières `period` valeurs sont `None` car il n'y a pas assez de données.
pub fn calculate_rsi(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
    if candles.len() < period + 1 {
        return vec![None; candles.len()];
    }

    let mut rsi_values = vec![None; period];
    let mut changes = Vec::new();

    // Calculer les changements de prix
    for i in 1..candles.len() {
        changes.push(candles[i].close - candles[i - 1].close);
    }

    // Calculer le RSI pour chaque période
    // Utiliser une plage inclusive pour produire une valeur par bougie (avec 'period' premières valeurs = None)
    for i in period..=changes.len() {
        let period_changes = &changes[i - period..i];
        
        let avg_gain = period_changes
            .iter()
            .filter(|&&c| c > 0.0)
            .sum::<f64>() / period as f64;
        
        let avg_loss = period_changes
            .iter()
            .filter(|&&c| c < 0.0)
            .map(|&c| -c)
            .sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            rsi_values.push(Some(100.0));
        } else {
            let rs = avg_gain / avg_loss;
            let rsi = 100.0 - (100.0 / (1.0 + rs));
            rsi_values.push(Some(rsi));
        }
    }

    rsi_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation() {
        // Créer des bougies de test avec une tendance haussière
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
            Candle::new(2000, 100.5, 102.0, 100.0, 101.0, 1000.0),
            Candle::new(3000, 101.0, 103.0, 101.0, 102.0, 1000.0),
            Candle::new(4000, 102.0, 104.0, 102.0, 103.0, 1000.0),
            Candle::new(5000, 103.0, 105.0, 103.0, 104.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2);
        // Avec seulement 5 bougies et une période de 2, on devrait avoir des valeurs RSI
        assert!(rsi.len() == candles.len());
    }

    #[test]
    fn test_macd_basic() {
        // Générer des bougies avec close croissant pour obtenir un MACD positif
        let mut candles = Vec::new();
        for i in 0..60 {
            let close = 100.0 + i as f64;
            candles.push(Candle::new(1000 + i * 1000, close, close + 1.0, close - 1.0, close, 1000.0));
        }

        let macd = calculate_macd(&candles, MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD);
        assert_eq!(macd.len(), candles.len());

        // Les premières valeurs (slow_period - 1) doivent être None
        for i in 0..(MACD_SLOW_PERIOD - 1) {
            assert!(macd[i].is_none(), "index {} should be None", i);
        }

        // La dernière valeur doit exister
        assert!(macd.last().and_then(|o| o.as_ref()).is_some());
        if let Some(Some(last)) = macd.last() {
            // Histogramme doit être fini
            assert!(last.histogram.is_finite());
        }
    }
}

/// Structure pour stocker les valeurs MACD
#[derive(Debug, Clone)]
pub struct MacdValue {
    pub macd_line: f64,      // MACD line (EMA rapide - EMA lente)
    pub signal_line: f64,    // Signal line (EMA de la ligne MACD)
    pub histogram: f64,      // Histogramme (MACD - Signal)
}

/// Calcule le MACD (Moving Average Convergence Divergence) pour une série de bougies
/// 
/// Le MACD est un indicateur de momentum qui montre la relation entre deux moyennes mobiles exponentielles.
/// 
/// # Arguments
/// * `candles` - Slice de bougies triées par timestamp croissant
/// * `fast_period` - Période pour l'EMA rapide (défaut: 12)
/// * `slow_period` - Période pour l'EMA lente (défaut: 26)
/// * `signal_period` - Période pour la ligne de signal (défaut: 9)
/// 
/// # Retourne
/// Un vecteur de valeurs MACD correspondant à chaque bougie.
/// Les premières valeurs sont `None` car il n'y a pas assez de données.
pub fn calculate_macd(
    candles: &[Candle],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Vec<Option<MacdValue>> {
    if candles.len() < slow_period + signal_period {
        return vec![None; candles.len()];
    }
    // Calculer les EMA rapide et lente
    let n = candles.len();
    let mut fast_ema: Vec<f64> = Vec::with_capacity(n);
    let mut slow_ema: Vec<f64> = Vec::with_capacity(n);

    // Pré-calcul des multiplicateurs
    let fast_mult = 2.0 / (fast_period + 1) as f64;
    let slow_mult = 2.0 / (slow_period + 1) as f64;

    // Calculer l'EMA rapide
    for i in 0..n {
        if i == 0 {
            // Valeur initiale brut
            fast_ema.push(candles[i].close);
        } else if i == fast_period - 1 {
            // Seed par la SMA des premières fast_period closes pour stabiliser l'EMA
            let sma = candles[0..fast_period].iter().map(|c| c.close).sum::<f64>() / fast_period as f64;
            fast_ema.push(sma);
        } else if i < fast_period - 1 {
            // Avant d'avoir assez de points pour la SMA, utiliser la close comme valeur
            fast_ema.push(candles[i].close);
        } else {
            let ema = (candles[i].close - fast_ema[i - 1]) * fast_mult + fast_ema[i - 1];
            fast_ema.push(ema);
        }
    }

    // Calculer l'EMA lente
    for i in 0..n {
        if i == 0 {
            slow_ema.push(candles[i].close);
        } else if i == slow_period - 1 {
            let sma = candles[0..slow_period].iter().map(|c| c.close).sum::<f64>() / slow_period as f64;
            slow_ema.push(sma);
        } else if i < slow_period - 1 {
            slow_ema.push(candles[i].close);
        } else {
            let ema = (candles[i].close - slow_ema[i - 1]) * slow_mult + slow_ema[i - 1];
            slow_ema.push(ema);
        }
    }

    // Calculer la ligne MACD (EMA rapide - EMA lente)
    let mut macd_line: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        macd_line.push(fast_ema[i] - slow_ema[i]);
    }

    // Calculer la ligne de signal (EMA de la ligne MACD)
    let mut signal_line: Vec<f64> = Vec::with_capacity(n);
    let signal_mult = 2.0 / (signal_period + 1) as f64;
    for i in 0..n {
        if i == 0 {
            signal_line.push(macd_line[i]);
        } else if i == signal_period - 1 {
            // Seed signal line with SMA of first signal_period macd_line values
            let sma = macd_line[0..signal_period].iter().sum::<f64>() / signal_period as f64;
            signal_line.push(sma);
        } else if i < signal_period - 1 {
            signal_line.push(macd_line[i]);
        } else {
            let ema = (macd_line[i] - signal_line[i - 1]) * signal_mult + signal_line[i - 1];
            signal_line.push(ema);
        }
    }
    
    // Construire le résultat
    let mut result: Vec<Option<MacdValue>> = Vec::with_capacity(n);
    // Préremplir les valeurs manquantes avant que la slow EMA soit disponible
    for _ in 0..(slow_period - 1) {
        result.push(None);
    }

    for i in (slow_period - 1)..n {
        let macd = macd_line[i];
        let signal = signal_line[i];
        let histogram = macd - signal;

        result.push(Some(MacdValue {
            macd_line: macd,
            signal_line: signal,
            histogram,
        }));
    }

    result
}

/// Rolling (incrémental) MACD calculator
/// Permet de mettre à jour le MACD en O(1) pour chaque nouvelle close
pub struct RollingMacd {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_mult: f64,
    slow_mult: f64,
    signal_mult: f64,
    closes: Vec<f64>,
    macd_history: Vec<f64>,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    prev_signal: Option<f64>,
}

impl RollingMacd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_mult: 2.0 / (fast_period + 1) as f64,
            slow_mult: 2.0 / (slow_period + 1) as f64,
            signal_mult: 2.0 / (signal_period + 1) as f64,
            closes: Vec::new(),
            macd_history: Vec::new(),
            prev_fast: None,
            prev_slow: None,
            prev_signal: None,
        }
    }

    /// Feed a new close price and return the MACD value for this index (or None if not enough data)
    pub fn feed(&mut self, close: f64) -> Option<MacdValue> {
        self.closes.push(close);
        let i = self.closes.len() - 1;

        // Compute fast EMA for index i
        let fast = if i == 0 {
            close
        } else if i == self.fast_period - 1 {
            let sma: f64 = self.closes[0..self.fast_period].iter().sum::<f64>() / self.fast_period as f64;
            sma
        } else if i < self.fast_period - 1 {
            close
        } else {
            let prev = self.prev_fast.expect("prev_fast should exist");
            (close - prev) * self.fast_mult + prev
        };
        self.prev_fast = Some(fast);

        // Compute slow EMA for index i
        let slow = if i == 0 {
            close
        } else if i == self.slow_period - 1 {
            let sma: f64 = self.closes[0..self.slow_period].iter().sum::<f64>() / self.slow_period as f64;
            sma
        } else if i < self.slow_period - 1 {
            close
        } else {
            let prev = self.prev_slow.expect("prev_slow should exist");
            (close - prev) * self.slow_mult + prev
        };
        self.prev_slow = Some(slow);

        // MACD line
        let macd_line = fast - slow;
        self.macd_history.push(macd_line);

        // Signal line
        let signal = if i == 0 {
            macd_line
        } else if i == self.signal_period - 1 {
            let sma: f64 = self.macd_history[0..self.signal_period].iter().sum::<f64>() / self.signal_period as f64;
            sma
        } else if i < self.signal_period - 1 {
            macd_line
        } else {
            let prev = self.prev_signal.expect("prev_signal should exist");
            (macd_line - prev) * self.signal_mult + prev
        };
        self.prev_signal = Some(signal);

        let histogram = macd_line - signal;

        if i >= self.slow_period - 1 {
            Some(MacdValue { macd_line, signal_line: signal, histogram })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod rolling_tests {
    use super::*;

    #[test]
    fn test_rolling_matches_full() {
        // Generate synthetic increasing closes
        let mut candles = Vec::new();
        for i in 0..200 {
            let close = 100.0 + (i as f64) * 0.5;
            candles.push(Candle::new(1000 + i as i64 * 1000, close, close + 0.1, close - 0.1, close, 1000.0));
        }

        let full = calculate_macd(&candles, MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD);

        let mut rolling = RollingMacd::new(MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD);
        let mut rolling_results: Vec<Option<MacdValue>> = Vec::with_capacity(candles.len());
        for c in &candles {
            rolling_results.push(rolling.feed(c.close));
        }

        assert_eq!(full.len(), rolling_results.len());
        for i in 0..full.len() {
            match (&full[i], &rolling_results[i]) {
                (None, None) => continue,
                (Some(a), Some(b)) => {
                    let eps = 1e-9;
                    assert!((a.macd_line - b.macd_line).abs() < eps, "macd_line mismatch at {}", i);
                    assert!((a.signal_line - b.signal_line).abs() < eps, "signal_line mismatch at {}", i);
                    assert!((a.histogram - b.histogram).abs() < eps, "histogram mismatch at {}", i);
                }
                _ => panic!("Mismatch at {}: full={:?}, rolling={:?}", i, full[i], rolling_results[i]),
            }
        }
    }
}
