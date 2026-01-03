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
#[derive(Debug, Clone, PartialEq)]
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
    
    // Vérifier que std_dev est valide (doit être >= 0)
    if std_dev < 0.0 || !std_dev.is_finite() {
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
        
        // Vérifier que la fenêtre a la bonne taille (devrait toujours être égale à period)
        let window_size = window.len();
        if window_size != period {
            // Ce cas ne devrait jamais se produire, mais on le gère pour la robustesse
            result.push(None);
            continue;
        }
        
        // Calculer la moyenne (SMA)
        let mean = window.iter().sum::<f64>() / window_size as f64;
        
        // Calculer la variance (écart-type de population, standard pour Bollinger Bands)
        let variance = window.iter()
            .map(|&price| {
                let diff = price - mean;
                diff * diff
            })
            .sum::<f64>() / window_size as f64;
        
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
                    1000.0, // volume
                )
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
                    1000.0, // volume
                )
            })
            .collect();
        
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        
        // Toutes les valeurs devraient être None car on n'a pas assez de données
        assert_eq!(result.len(), 10);
        for val in result {
            assert_eq!(val, None);
        }
    }
    
    #[test]
    fn test_bollinger_bands_known_values() {
        // Test avec des valeurs connues pour vérifier la formule
        // Pour une série: [100, 102, 101, 103, 102, 104, 103, 105, 104, 106, 
        //                  105, 107, 106, 108, 107, 109, 108, 110, 109, 111]
        // Avec period=20, std_dev=2.0
        
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 102.0, 104.0, 103.0, 105.0, 104.0, 106.0,
            105.0, 107.0, 106.0, 108.0, 107.0, 109.0, 108.0, 110.0, 109.0, 111.0,
        ];
        
        let candles: Vec<Candle> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| {
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
        
        let result = calculate_bollinger_bands(&candles, 20, 2.0);
        
        // Vérifier que la première valeur calculée existe
        assert!(result[19].is_some());
        
        if let Some(bb) = &result[19] {
            // Calcul manuel pour vérification:
            // Moyenne = (100+102+...+111) / 20 = 105.5
            let expected_mean = prices.iter().sum::<f64>() / 20.0;
            assert!((bb.middle - expected_mean).abs() < 0.001);
            
            // Vérifier que les bandes sont symétriques autour de la moyenne
            let band_width = (bb.upper - bb.lower) / 2.0;
            assert!((bb.upper - bb.middle - band_width).abs() < 0.001);
            assert!((bb.middle - bb.lower - band_width).abs() < 0.001);
            
            // Vérifier que upper > middle > lower
            assert!(bb.upper > bb.middle);
            assert!(bb.middle > bb.lower);
        }
    }
    
    #[test]
    fn test_bollinger_bands_window_size() {
        // Vérifier que la fenêtre glissante a toujours la bonne taille
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let price = 100.0 + (i as f64 * 0.5);
                Candle::new(
                    1000 + i * 60,
                    price,
                    price,
                    price,
                    price,
                    1000.0,
                )
            })
            .collect();
        
        let period = 20;
        let result = calculate_bollinger_bands(&candles, period, 2.0);
        
        // Vérifier que toutes les valeurs calculées ont des bandes valides
        for (i, bb_opt) in result.iter().enumerate() {
            if let Some(bb) = bb_opt {
                // Vérifier que les bandes sont dans le bon ordre
                assert!(bb.upper >= bb.middle, "Index {}: upper ({}) < middle ({})", i, bb.upper, bb.middle);
                assert!(bb.middle >= bb.lower, "Index {}: middle ({}) < lower ({})", i, bb.middle, bb.lower);
                
                // Vérifier que les bandes ne sont pas infinies ou NaN
                assert!(bb.upper.is_finite(), "Index {}: upper is not finite", i);
                assert!(bb.middle.is_finite(), "Index {}: middle is not finite", i);
                assert!(bb.lower.is_finite(), "Index {}: lower is not finite", i);
            }
        }
    }
    
    #[test]
    fn test_bollinger_bands_zero_std_dev() {
        // Test avec std_dev = 0 (les bandes devraient être égales à la moyenne)
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                Candle::new(
                    1000 + i * 60,
                    100.0,
                    100.0,
                    100.0,
                    100.0,
                    1000.0,
                )
            })
            .collect();
        
        let result = calculate_bollinger_bands(&candles, 20, 0.0);
        
        if let Some(bb) = &result[19] {
            // Avec std_dev = 0, toutes les bandes devraient être égales
            assert!((bb.upper - bb.middle).abs() < 0.001);
            assert!((bb.middle - bb.lower).abs() < 0.001);
        }
    }
}


