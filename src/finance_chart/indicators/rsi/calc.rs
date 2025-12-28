//! Calculs du RSI (Relative Strength Index)
//!
//! Le RSI est un indicateur de momentum qui mesure la vitesse et l'amplitude
//! des variations de prix. Il varie entre 0 et 100.

use crate::finance_chart::core::Candle;

/// Paramètres par défaut pour le RSI
pub const RSI_PERIOD: usize = 14;
pub const RSI_OVERBOUGHT: f64 = 70.0;
pub const RSI_OVERSOLD: f64 = 30.0;

/// Calcule le RSI (Relative Strength Index) pour une série de bougies
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
}

