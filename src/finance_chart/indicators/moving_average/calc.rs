//! Calculs de la moyenne mobile (Simple Moving Average - SMA)
//!
//! La moyenne mobile est calculée en faisant la moyenne arithmétique
//! des prix de clôture sur une période donnée.

use crate::finance_chart::core::Candle;

/// Période par défaut pour la moyenne mobile
pub const MA_PERIOD: usize = 20;

/// Calcule la moyenne mobile (SMA) pour une série de bougies
/// 
/// # Arguments
/// * `candles` - Slice de bougies triées par timestamp croissant
/// * `period` - Période pour la SMA (défaut: 20)
/// 
/// # Retourne
/// Un vecteur de valeurs SMA correspondant à chaque bougie.
/// Les premières valeurs sont `None` car il n'y a pas assez de données.
pub fn calculate_moving_average(
    candles: &[Candle],
    period: usize,
) -> Vec<Option<f64>> {
    // Vérifier que la période est valide (au moins 1)
    if period == 0 {
        return vec![None; candles.len()];
    }
    
    if candles.len() < period {
        return vec![None; candles.len()];
    }

    let n = candles.len();
    let mut result: Vec<Option<f64>> = Vec::with_capacity(n);
    
    // Préremplir les valeurs manquantes avant que la période soit complète
    // Utiliser checked_sub pour éviter les débordements
    if let Some(prefill_count) = period.checked_sub(1) {
        for _ in 0..prefill_count {
            result.push(None);
        }
    }
    
    // Calculer la SMA pour chaque position à partir de la période
    // Utiliser checked_sub pour éviter les débordements
    let start_idx = match period.checked_sub(1) {
        Some(idx) if idx < n => idx,
        _ => {
            // Si period - 1 >= n, on ne peut rien calculer
            return result;
        }
    };
    
    for i in start_idx..n {
        // Extraire les prix de clôture pour la fenêtre glissante
        // Utiliser checked_sub pour éviter les débordements
        let window_start = match period.checked_sub(1).and_then(|p| i.checked_sub(p)) {
            Some(idx) => idx,
            None => {
                // Si le calcul échoue, on ne peut pas calculer cette valeur
                result.push(None);
                continue;
            }
        };
        
        // Calculer la moyenne (SMA)
        let sum: f64 = candles[window_start..=i]
            .iter()
            .map(|c| c.close)
            .sum();
        
        let ma_value = sum / period as f64;
        result.push(Some(ma_value));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average_basic() {
        // Créer des bougies de test avec des prix constants
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                Candle::new(
                    (1000 + i * 60) as i64,
                    100.0,
                    100.0,
                    100.0,
                    100.0,
                    1000.0,
                )
            })
            .collect();
        
        let result = calculate_moving_average(&candles, 20);
        
        // Vérifier que les premières valeurs sont None
        assert_eq!(result[0], None);
        assert_eq!(result[18], None);
        
        // Vérifier que les valeurs après la période sont calculées
        assert!(result[19].is_some());
        
        // Pour des prix constants, la moyenne devrait être 100.0
        if let Some(ma) = result[19] {
            assert!((ma - 100.0).abs() < 0.01);
        }
    }
    
    #[test]
    fn test_moving_average_insufficient_data() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| {
                Candle::new(
                    (1000 + i * 60) as i64,
                    100.0,
                    100.0,
                    100.0,
                    100.0,
                    1000.0,
                )
            })
            .collect();
        
        let result = calculate_moving_average(&candles, 20);
        
        // Toutes les valeurs devraient être None car on n'a pas assez de données
        assert_eq!(result.len(), 10);
        for val in result {
            assert_eq!(val, None);
        }
    }
    
    #[test]
    fn test_moving_average_increasing() {
        // Créer des bougies avec des prix croissants
        let candles: Vec<Candle> = (0..25)
            .map(|i| {
                let price = 100.0 + i as f64;
                Candle::new(
                    (1000 + i * 60) as i64,
                    price,
                    price,
                    price,
                    price,
                    1000.0,
                )
            })
            .collect();
        
        let result = calculate_moving_average(&candles, 20);
        
        // La première valeur calculée devrait être la moyenne de [0..19]
        if let Some(ma) = result[19] {
            let expected: f64 = (0..20).map(|i| 100.0 + i as f64).sum::<f64>() / 20.0;
            assert!((ma - expected).abs() < 0.01);
        }
        
        // La deuxième valeur devrait être la moyenne de [1..20]
        if let Some(ma) = result[20] {
            let expected: f64 = (1..21).map(|i| 100.0 + i as f64).sum::<f64>() / 20.0;
            assert!((ma - expected).abs() < 0.01);
        }
    }
}

