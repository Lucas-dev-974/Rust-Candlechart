//! Calculs du RSI (Relative Strength Index)
//!
//! Le RSI est un indicateur de momentum qui mesure la vitesse et l'amplitude
//! des variations de prix. Il varie entre 0 et 100.

use crate::finance_chart::core::Candle;
use crate::app::state::RSIMethod;

/// Paramètres par défaut pour le RSI
pub const RSI_PERIOD: usize = 14;
pub const RSI_OVERBOUGHT: f64 = 70.0;
pub const RSI_OVERSOLD: f64 = 30.0;

/// Calcule le RSI (Relative Strength Index) pour une série de bougies
/// 
/// Supporte deux méthodes :
/// - Wilder : Méthode standard de Welles Wilder avec moyenne mobile lissée
/// - Simple : Moyenne mobile simple (SMA) pour chaque fenêtre
/// 
/// # Arguments
/// * `candles` - Slice de bougies triées par timestamp croissant
/// * `period` - Période pour le calcul (défaut: 14)
/// * `method` - Méthode de calcul (Wilder ou Simple)
/// 
/// # Retourne
/// Un vecteur de valeurs RSI correspondant à chaque bougie.
/// Les premières `period` valeurs sont `None` car il n'y a pas assez de données.
pub fn calculate_rsi(candles: &[Candle], period: usize, method: RSIMethod) -> Vec<Option<f64>> {
    if candles.len() < period + 1 {
        return vec![None; candles.len()];
    }

    let mut rsi_values = vec![None; period];
    let mut changes = Vec::new();

    // Calculer les changements de prix
    for i in 1..candles.len() {
        changes.push(candles[i].close - candles[i - 1].close);
    }

    match method {
        RSIMethod::Simple => {
            // Méthode Simple : SMA glissante pour chaque fenêtre
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

                // Gérer les cas limites
                if avg_loss == 0.0 {
                    if avg_gain == 0.0 {
                        // Pas de mouvement : RSI neutre
                        rsi_values.push(Some(50.0));
                    } else {
                        // Seulement des gains : RSI maximum
                        rsi_values.push(Some(100.0));
                    }
                } else {
                    let rs = avg_gain / avg_loss;
                    let rsi = 100.0 - (100.0 / (1.0 + rs));
                    rsi_values.push(Some(rsi));
                }
            }
        }
        RSIMethod::Wilder => {
            // Méthode Wilder : moyenne mobile lissée
            // Première période : moyenne simple
            let first_period_changes = &changes[0..period];
            let mut avg_gain = first_period_changes
                .iter()
                .filter(|&&c| c > 0.0)
                .sum::<f64>() / period as f64;
            let mut avg_loss = first_period_changes
                .iter()
                .filter(|&&c| c < 0.0)
                .map(|&c| -c)
                .sum::<f64>() / period as f64;

            // Calculer le premier RSI
            if avg_loss == 0.0 {
                if avg_gain == 0.0 {
                    // Pas de mouvement : RSI neutre
                    rsi_values.push(Some(50.0));
                } else {
                    // Seulement des gains : RSI maximum
                    rsi_values.push(Some(100.0));
                }
            } else {
                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));
                rsi_values.push(Some(rsi));
            }

            // Périodes suivantes : moyenne lissée (Wilder's smoothing)
            for i in period..changes.len() {
                let current_change = changes[i];
                let current_gain = if current_change > 0.0 { current_change } else { 0.0 };
                let current_loss = if current_change < 0.0 { -current_change } else { 0.0 };

                // Wilder's smoothing: (avg_prev * (period - 1) + current) / period
                avg_gain = (avg_gain * (period - 1) as f64 + current_gain) / period as f64;
                avg_loss = (avg_loss * (period - 1) as f64 + current_loss) / period as f64;

                // Gérer les cas limites
                if avg_loss == 0.0 {
                    if avg_gain == 0.0 {
                        // Pas de mouvement : RSI neutre
                        rsi_values.push(Some(50.0));
                    } else {
                        // Seulement des gains : RSI maximum
                        rsi_values.push(Some(100.0));
                    }
                } else {
                    let rs = avg_gain / avg_loss;
                    let rsi = 100.0 - (100.0 / (1.0 + rs));
                    rsi_values.push(Some(rsi));
                }
            }
        }
    }

    rsi_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation_simple() {
        // Créer des bougies de test avec une tendance haussière
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
            Candle::new(2000, 100.5, 102.0, 100.0, 101.0, 1000.0),
            Candle::new(3000, 101.0, 103.0, 101.0, 102.0, 1000.0),
            Candle::new(4000, 102.0, 104.0, 102.0, 103.0, 1000.0),
            Candle::new(5000, 103.0, 105.0, 103.0, 104.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Simple);
        // Avec seulement 5 bougies et une période de 2, on devrait avoir des valeurs RSI
        assert_eq!(rsi.len(), candles.len());
        // Les 2 premières valeurs doivent être None
        assert!(rsi[0].is_none());
        assert!(rsi[1].is_none());
        // Les valeurs suivantes doivent être Some
        assert!(rsi[2].is_some());
    }

    #[test]
    fn test_rsi_calculation_wilder() {
        // Créer des bougies de test avec une tendance haussière
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
            Candle::new(2000, 100.5, 102.0, 100.0, 101.0, 1000.0),
            Candle::new(3000, 101.0, 103.0, 101.0, 102.0, 1000.0),
            Candle::new(4000, 102.0, 104.0, 102.0, 103.0, 1000.0),
            Candle::new(5000, 103.0, 105.0, 103.0, 104.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Wilder);
        assert_eq!(rsi.len(), candles.len());
        // Les 2 premières valeurs doivent être None
        assert!(rsi[0].is_none());
        assert!(rsi[1].is_none());
        // Les valeurs suivantes doivent être Some
        assert!(rsi[2].is_some());
    }

    #[test]
    fn test_rsi_all_gains() {
        // Toutes les bougies montent
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 101.0, 1000.0),
            Candle::new(2000, 101.0, 102.0, 100.0, 102.0, 1000.0),
            Candle::new(3000, 102.0, 103.0, 101.0, 103.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Simple);
        // Avec seulement des gains, RSI devrait être proche de 100
        if let Some(rsi_value) = rsi[2] {
            assert!(rsi_value >= 50.0 && rsi_value <= 100.0);
        }
    }

    #[test]
    fn test_rsi_all_losses() {
        // Toutes les bougies baissent
        let candles = vec![
            Candle::new(1000, 103.0, 104.0, 102.0, 103.0, 1000.0),
            Candle::new(2000, 103.0, 104.0, 101.0, 102.0, 1000.0),
            Candle::new(3000, 102.0, 103.0, 100.0, 101.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Simple);
        // Avec seulement des pertes, RSI devrait être proche de 0
        if let Some(rsi_value) = rsi[2] {
            assert!(rsi_value >= 0.0 && rsi_value <= 50.0);
        }
    }

    #[test]
    fn test_rsi_no_movement() {
        // Toutes les bougies ont le même prix de clôture
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.0, 1000.0),
            Candle::new(2000, 100.0, 101.0, 99.0, 100.0, 1000.0),
            Candle::new(3000, 100.0, 101.0, 99.0, 100.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Simple);
        // Sans mouvement, RSI devrait être 50.0 (neutre)
        if let Some(rsi_value) = rsi[2] {
            assert_eq!(rsi_value, 50.0, "RSI devrait être 50.0 quand il n'y a pas de mouvement");
        }
    }

    #[test]
    fn test_rsi_insufficient_data() {
        // Pas assez de données pour calculer le RSI
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 14, RSIMethod::Simple);
        // Toutes les valeurs doivent être None
        assert_eq!(rsi.len(), 1);
        assert!(rsi[0].is_none());
    }

    #[test]
    fn test_rsi_bounds() {
        // Vérifier que RSI est toujours entre 0 et 100
        let candles = vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
            Candle::new(2000, 100.5, 102.0, 100.0, 101.0, 1000.0),
            Candle::new(3000, 101.0, 103.0, 101.0, 102.0, 1000.0),
            Candle::new(4000, 102.0, 104.0, 102.0, 103.0, 1000.0),
            Candle::new(5000, 103.0, 105.0, 103.0, 104.0, 1000.0),
        ];

        let rsi = calculate_rsi(&candles, 2, RSIMethod::Simple);
        for rsi_opt in rsi.iter() {
            if let Some(rsi_value) = rsi_opt {
                assert!(*rsi_value >= 0.0 && *rsi_value <= 100.0, 
                    "RSI devrait être entre 0 et 100, mais a obtenu {}", rsi_value);
            }
        }
    }
}

