//! Fonctions helper pour le calcul et l'extraction des données de la moyenne mobile
//!
//! Ce module contient la logique partagée pour calculer la moyenne mobile avec toutes les bougies
//! et extraire les valeurs correspondant aux bougies visibles.

use crate::finance_chart::state::ChartState;
use crate::finance_chart::core::Candle;
use super::calc::{calculate_moving_average, MA_PERIOD};

/// Calcule la moyenne mobile pour toutes les bougies et retourne les valeurs correspondant aux bougies visibles
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `all_ma_values` - Toutes les valeurs MA pré-calculées
///
/// # Retourne
/// Un tuple contenant :
/// - Les valeurs MA correspondant aux bougies visibles
/// - Les bougies visibles
/// - L'index de début des bougies visibles
pub fn calculate_ma_data<'a>(
    chart_state: &'a ChartState,
    all_ma_values: &'a [Option<f64>],
) -> Option<(&'a [Option<f64>], &'a [Candle], usize)> {
    if all_ma_values.is_empty() {
        return None;
    }

    // Récupérer les bougies visibles pour déterminer quelle partie de la MA afficher
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
    let end = (visible_start_idx + visible_candles_slice.len()).min(all_ma_values.len());
    let slice = &all_ma_values[visible_start_idx..end];

    Some((slice, visible_candles_slice, visible_start_idx))
}

/// Calcule toutes les valeurs de la moyenne mobile pour toutes les bougies
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `period` - Période pour le calcul (optionnel, utilise MA_PERIOD par défaut)
///
/// # Retourne
/// Toutes les valeurs MA calculées, ou `None` si le calcul n'est pas possible
pub fn calculate_all_ma_values(
    chart_state: &ChartState,
    period: Option<usize>,
) -> Option<Vec<Option<f64>>> {
    let all_candles = chart_state.all_candles()?;
    
    if all_candles.is_empty() {
        return None;
    }

    let period = period.unwrap_or(MA_PERIOD);

    let all_ma_values = calculate_moving_average(
        all_candles,
        period,
    );
    
    if all_ma_values.is_empty() {
        None
    } else {
        Some(all_ma_values)
    }
}

