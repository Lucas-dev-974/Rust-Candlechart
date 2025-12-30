//! Calculs des bandes de Bollinger
//!
//! Les bandes de Bollinger sont composées de :
//! - Une bande moyenne (SMA - Simple Moving Average)
//! - Une bande supérieure (moyenne + 2 * écart-type)
//! - Une bande inférieure (moyenne - 2 * écart-type)

use crate::finance_chart::core::Candle;

/// Paramètres par défaut pour les bandes de Bollinger
pub const BOLLINGER_PERIOD: usize = 20;
pub const BOLLINGER_STD_DEV: f64 = 2.0;

/// Structure pour stocker les valeurs des bandes de Bollinger
#[derive(Debug, Clone)]
pub struct BollingerValue {
    pub middle: f64,  // Bande moyenne (SMA)
    pub upper: f64,   // Bande supérieure
    pub lower: f64,   // Bande inférieure
}

/// Calcule les bandes de Bollinger pour une série de bougies
/// 
/// # Arguments
/// * `candles` - Slice de bougies triées par timestamp croissant
/// * `period` - Période pour la SMA et l'écart-type (défaut: 20)
/// * `std_dev` - Nombre d'écarts-types pour les bandes (défaut: 2.0)
/// 
/// # Retourne
/// Un vecteur de valeurs Bollinger correspondant à chaque bougie.
/// Les premières valeurs sont `None` car il n'y a pas assez de données.
pub fn calculate_bollinger_bands(
    candles: &[Candle],
    period: usize,
    std_dev: f64,
) -> Vec<Option<BollingerValue>> {
    // Vérifier que la période est valide (au moins 1)
    if period == 0 {
        return vec![None; candles.len()];
    }
    
    if candles.len() < period {
        return vec![None; candles.len()];
    }

    let n = candles.len();
    let mut result: Vec<Option<BollingerValue>> = Vec::with_capacity(n);
    
    // Préremplir les valeurs manquantes avant que la période soit complète
    // Utiliser checked_sub pour éviter les débordements
    if let Some(prefill_count) = period.checked_sub(1) {
        for _ in 0..prefill_count {
            result.push(None);
        }
    }
    
    // Calculer les bandes pour chaque position à partir de la période
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
        
        let window: Vec<f64> = candles[window_start..=i]
            .iter()
            .map(|c| c.close)
            .collect();
        
        // Calculer la moyenne (SMA)
        let mean = window.iter().sum::<f64>() / period as f64;
        
        // Calculer la variance
        let variance = window.iter()
            .map(|&price| {
                let diff = price - mean;
                diff * diff
            })
            .sum::<f64>() / period as f64;
        
        // Calculer l'écart-type
        let std = variance.sqrt();
        
        // Calculer les bandes
        let upper = mean + (std_dev * std);
        let lower = mean - (std_dev * std);
        
        result.push(Some(BollingerValue {
            middle: mean,
            upper,
            lower,
        }));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance_chart::core::Candle;

    #[test]
    fn test_bollinger_bands_basic() {
        // Créer des bougies de test avec des prix constants
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                Candle::new(
                    1000 + i * 60,
                    100.0,
                    100.0,
                    100.0,
                    100.0,
                ).unwrap()
            })
            .collect();
        
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        
        // Vérifier que les premières valeurs sont None
        assert_eq!(result[0], None);
        assert_eq!(result[18], None);
        
        // Vérifier que les valeurs après la période sont calculées
        assert!(result[19].is_some());
        
        // Pour des prix constants, la moyenne devrait être 100.0 et l'écart-type proche de 0
        if let Some(bb) = &result[19] {
            assert!((bb.middle - 100.0).abs() < 0.01);
            assert!((bb.upper - 100.0).abs() < 0.01);
            assert!((bb.lower - 100.0).abs() < 0.01);
        }
    }
    
    #[test]
    fn test_bollinger_bands_insufficient_data() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| {
                Candle::new(
                    1000 + i * 60,
                    100.0,
                    100.0,
                    100.0,
                    100.0,
                ).unwrap()
            })
            .collect();
        
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        
        // Toutes les valeurs devraient être None car on n'a pas assez de données
        assert_eq!(result.len(), 10);
        for val in result {
            assert_eq!(val, None);
        }
    }
}


