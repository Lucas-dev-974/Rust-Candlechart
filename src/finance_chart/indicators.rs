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
    for i in period..changes.len() {
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
    let mut fast_ema = Vec::new();
    let mut slow_ema = Vec::new();
    
    // Calculer l'EMA rapide
    for i in 0..candles.len() {
        if i == 0 {
            fast_ema.push(candles[i].close);
        } else {
            let multiplier = 2.0 / (fast_period + 1) as f64;
            let ema = (candles[i].close - fast_ema[i - 1]) * multiplier + fast_ema[i - 1];
            fast_ema.push(ema);
        }
    }
    
    // Calculer l'EMA lente
    for i in 0..candles.len() {
        if i == 0 {
            slow_ema.push(candles[i].close);
        } else {
            let multiplier = 2.0 / (slow_period + 1) as f64;
            let ema = (candles[i].close - slow_ema[i - 1]) * multiplier + slow_ema[i - 1];
            slow_ema.push(ema);
        }
    }
    
    // Calculer la ligne MACD (EMA rapide - EMA lente)
    let mut macd_line = Vec::new();
    for i in 0..candles.len() {
        macd_line.push(fast_ema[i] - slow_ema[i]);
    }
    
    // Calculer la ligne de signal (EMA de la ligne MACD)
    let mut signal_line = Vec::new();
    for i in 0..macd_line.len() {
        if i == 0 {
            signal_line.push(macd_line[i]);
        } else {
            let multiplier = 2.0 / (signal_period + 1) as f64;
            let ema = (macd_line[i] - signal_line[i - 1]) * multiplier + signal_line[i - 1];
            signal_line.push(ema);
        }
    }
    
    // Construire le résultat
    let mut result = vec![None; slow_period - 1]; // Pas assez de données pour l'EMA lente
    
    for i in (slow_period - 1)..candles.len() {
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
