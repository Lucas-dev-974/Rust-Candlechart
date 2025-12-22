//! Calcul de la largeur des barres (bougies et volumes)
//!
//! Module utilitaire pour factoriser le calcul de la largeur des barres
//! entre le graphique de bougies et le graphique de volumes.

use super::super::core::Candle;

/// Constantes pour le dimensionnement des barres
pub const MIN_GAP: f32 = 3.0;      // Espacement minimum entre barres
pub const MAX_WIDTH: f32 = 20.0;   // Largeur maximum d'une barre
pub const MIN_WIDTH: f32 = 1.0;    // Largeur minimum d'une barre

/// Calcule la période (intervalle de temps) entre les bougies
///
/// Utilise la médiane des intervalles pour être robuste aux variations/outliers.
pub fn calculate_candle_period(candles: &[Candle]) -> i64 {
    if candles.len() < 2 {
        return 3600; // Par défaut 1 heure
    }

    // Calculer plusieurs intervalles et prendre la médiane
    let mut intervals: Vec<i64> = Vec::new();
    for i in 1..candles.len().min(10) {
        let interval = (candles[i].timestamp - candles[i - 1].timestamp).abs();
        if interval > 0 {
            intervals.push(interval);
        }
    }

    if intervals.is_empty() {
        3600 // Fallback
    } else {
        intervals.sort();
        intervals[intervals.len() / 2] // Médiane
    }
}

/// Calcule le ratio de largeur adaptatif basé sur l'espacement en pixels
fn calculate_width_ratio(spacing: f32) -> f32 {
    if spacing > 25.0 {
        1.0  // Très zoom in : barres pleine largeur (100%)
    } else if spacing > 15.0 {
        0.8  // Zoom in : barres larges (80%)
    } else if spacing > 8.0 {
        0.6  // Zoom moyen : barres moyennes (60%)
    } else if spacing > 5.0 {
        0.4  // Zoom out : barres fines (40%)
    } else {
        0.3  // Zoom très out : barres très fines (30%)
    }
}

/// Calcule la largeur des barres en pixels
///
/// # Arguments
/// * `candle_period` - Intervalle de temps entre bougies en secondes
/// * `time_range` - Plage de temps visible (max_time - min_time)
/// * `viewport_width` - Largeur du viewport en pixels
///
/// # Retourne
/// La largeur en pixels pour les barres (bougies ou volumes)
pub fn calculate_bar_width(candle_period: i64, time_range: i64, viewport_width: f32) -> f32 {
    let time_range_f64 = time_range as f64;
    let pixels_per_second = viewport_width as f64 / time_range_f64;
    let spacing = (candle_period as f64 * pixels_per_second) as f32;

    let width_ratio = calculate_width_ratio(spacing);

    (spacing * width_ratio)
        .min(spacing - MIN_GAP)
        .clamp(MIN_WIDTH, MAX_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_candle_period_empty() {
        let candles: Vec<Candle> = vec![];
        assert_eq!(calculate_candle_period(&candles), 3600);
    }

    #[test]
    fn test_calculate_candle_period_single() {
        let candles = vec![Candle::new(1000, 100.0, 105.0, 99.0, 104.0, 100.0)];
        assert_eq!(calculate_candle_period(&candles), 3600);
    }

    #[test]
    fn test_calculate_candle_period_multiple() {
        let candles = vec![
            Candle::new(1000, 100.0, 105.0, 99.0, 104.0, 100.0),
            Candle::new(1900, 100.0, 105.0, 99.0, 104.0, 100.0), // 900s interval
            Candle::new(2800, 100.0, 105.0, 99.0, 104.0, 100.0), // 900s interval
        ];
        assert_eq!(calculate_candle_period(&candles), 900);
    }

    #[test]
    fn test_calculate_bar_width() {
        // 1 heure de période, 24h de range, 800px de largeur
        let width = calculate_bar_width(3600, 86400, 800.0);
        assert!(width >= MIN_WIDTH && width <= MAX_WIDTH);
    }
}

