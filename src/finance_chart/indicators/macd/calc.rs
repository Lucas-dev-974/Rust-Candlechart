//! Calculs du MACD (Moving Average Convergence Divergence)
//!
//! Le MACD est un indicateur de momentum qui montre la relation entre
//! deux moyennes mobiles exponentielles.

use crate::finance_chart::core::Candle;
use super::super::ema::Ema;

/// Paramètres par défaut pour le MACD
pub const MACD_FAST_PERIOD: usize = 12;
pub const MACD_SLOW_PERIOD: usize = 26;
pub const MACD_SIGNAL_PERIOD: usize = 9;

/// Structure pour stocker les valeurs MACD
#[derive(Debug, Clone)]
pub struct MacdValue {
    pub macd_line: f64,      // MACD line (EMA rapide - EMA lente)
    pub signal_line: f64,    // Signal line (EMA de la ligne MACD)
    pub histogram: f64,      // Histogramme (MACD - Signal)
}

/// Calcule le MACD (Moving Average Convergence Divergence) pour une série de bougies
/// 
/// Utilise le module EMA réutilisable pour les calculs.
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

    let n = candles.len();
    
    // Utiliser les calculateurs EMA du module ema
    let mut fast_ema_calc = Ema::new(fast_period);
    let mut slow_ema_calc = Ema::new(slow_period);
    let mut signal_ema_calc = Ema::new(signal_period);

    // Calculer les EMA rapide et lente
    let mut fast_ema: Vec<f64> = Vec::with_capacity(n);
    let mut slow_ema: Vec<f64> = Vec::with_capacity(n);
    
    for candle in candles {
        fast_ema.push(fast_ema_calc.feed(candle.close));
        slow_ema.push(slow_ema_calc.feed(candle.close));
    }

    // Calculer la ligne MACD (EMA rapide - EMA lente)
    let macd_line: Vec<f64> = fast_ema.iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| f - s)
        .collect();

    // Calculer la ligne de signal (EMA de la ligne MACD)
    let mut signal_line: Vec<f64> = Vec::with_capacity(n);
    for &macd_val in &macd_line {
        signal_line.push(signal_ema_calc.feed(macd_val));
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

#[cfg(test)]
mod tests {
    use super::*;

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

