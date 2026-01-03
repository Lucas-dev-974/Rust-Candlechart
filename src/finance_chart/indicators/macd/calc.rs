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
    // IMPORTANT: La ligne de signal ne doit être calculée qu'à partir du moment
    // où la ligne MACD est valide (après slow_period - 1 valeurs)
    let mut signal_line: Vec<f64> = Vec::with_capacity(n);
    
    // Les premières valeurs de la ligne MACD ne sont pas valides avant slow_period - 1
    // On ne commence à calculer la ligne de signal qu'à partir de là
    for i in 0..n {
        if i < slow_period - 1 {
            // Avant que la ligne MACD soit valide, on ne peut pas calculer la ligne de signal
            signal_line.push(0.0); // Valeur temporaire, ne sera pas utilisée
        } else {
            // À partir de slow_period - 1, la ligne MACD est valide, on peut calculer la ligne de signal
            signal_line.push(signal_ema_calc.feed(macd_line[i]));
        }
    }
    
    // Construire le résultat
    let mut result: Vec<Option<MacdValue>> = Vec::with_capacity(n);
    
    // Le premier index valide est après que :
    // 1. La slow EMA soit disponible (slow_period - 1)
    // 2. La ligne de signal soit disponible (signal_period valeurs de MACD valide)
    // Donc: slow_period - 1 + signal_period - 1 = slow_period + signal_period - 2
    let first_valid_index = (slow_period - 1) + (signal_period - 1);
    
    // Préremplir les valeurs manquantes avant que toutes les données soient disponibles
    for _ in 0..first_valid_index {
        result.push(None);
    }

    // À partir de first_valid_index, toutes les valeurs sont valides
    for i in first_valid_index..n {
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

        // Le premier index valide est après slow_period + signal_period - 2
        // car on a besoin de slow_period pour la ligne MACD et signal_period pour la ligne de signal
        let first_valid_index = (MACD_SLOW_PERIOD - 1) + (MACD_SIGNAL_PERIOD - 1);
        
        // Les premières valeurs doivent être None jusqu'à ce que toutes les données soient disponibles
        for i in 0..first_valid_index {
            assert!(macd[i].is_none(), "index {} should be None", i);
        }

        // À partir de first_valid_index, les valeurs doivent exister
        assert!(macd[first_valid_index].is_some(), "index {} should have a value", first_valid_index);

        // La dernière valeur doit exister
        assert!(macd.last().and_then(|o| o.as_ref()).is_some());
        if let Some(Some(last)) = macd.last() {
            // Histogramme doit être fini
            assert!(last.histogram.is_finite());
            // Les valeurs MACD et signal doivent être finies
            assert!(last.macd_line.is_finite());
            assert!(last.signal_line.is_finite());
        }
    }
}

