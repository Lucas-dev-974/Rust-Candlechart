//! Fonctions helper pour le calcul et l'extraction des données des bandes de Bollinger
//!
//! Ce module contient la logique partagée pour calculer les bandes de Bollinger avec toutes les bougies
//! et extraire les valeurs correspondant aux bougies visibles.

use crate::finance_chart::state::ChartState;
use crate::finance_chart::core::Candle;
use super::calc::{calculate_bollinger_bands, BollingerValue, BOLLINGER_PERIOD, BOLLINGER_STD_DEV};

/// Calcule les bandes de Bollinger pour toutes les bougies et retourne les valeurs correspondant aux bougies visibles
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `all_bollinger_values` - Toutes les valeurs Bollinger pré-calculées
///
/// # Retourne
/// Un tuple contenant :
/// - Les valeurs Bollinger correspondant aux bougies visibles
/// - Les bougies visibles
/// - L'index de début des bougies visibles
pub fn calculate_bollinger_data<'a>(
    chart_state: &'a ChartState,
    all_bollinger_values: &'a [Option<BollingerValue>],
) -> Option<(&'a [Option<BollingerValue>], &'a [Candle], usize)> {
    if all_bollinger_values.is_empty() {
        return None;
    }

    // Récupérer les bougies visibles pour déterminer quelle partie des bandes afficher
    let visible_candles = chart_state.visible_candles();
    if visible_candles.is_empty() {
        return None;
    }
    let (_, visible_candles_slice) = &visible_candles[0];

    // Récupérer toutes les bougies pour trouver l'index de début
    let all_candles = chart_state.all_candles()?;

    // Trouver l'index de début des bougies visibles dans toutes les bougies
    let visible_start_idx = if let Some(first_visible) = visible_candles_slice.first() {
        all_candles
            .iter()
            .position(|c| c.timestamp == first_visible.timestamp)
            .unwrap_or(0)
    } else {
        0
    };

    // Limiter la tranche pour éviter un out-of-bounds si les vecteurs diffèrent
    let end = (visible_start_idx + visible_candles_slice.len()).min(all_bollinger_values.len());
    let slice = &all_bollinger_values[visible_start_idx..end];

    Some((slice, visible_candles_slice, visible_start_idx))
}

/// Calcule toutes les valeurs des bandes de Bollinger pour toutes les bougies
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
///
/// # Retourne
/// Toutes les valeurs Bollinger calculées, ou `None` si le calcul n'est pas possible
pub fn calculate_all_bollinger_values(
    chart_state: &ChartState,
    period: Option<usize>,
    std_dev: Option<f64>,
) -> Option<Vec<Option<BollingerValue>>> {
    let all_candles = chart_state.all_candles()?;
    
    if all_candles.is_empty() {
        return None;
    }

    let period = period.unwrap_or(BOLLINGER_PERIOD);
    let std_dev = std_dev.unwrap_or(BOLLINGER_STD_DEV);

    let all_bollinger_values = calculate_bollinger_bands(
        all_candles,
        period,
        std_dev,
    );
    
    if all_bollinger_values.is_empty() {
        None
    } else {
        Some(all_bollinger_values)
    }
}

