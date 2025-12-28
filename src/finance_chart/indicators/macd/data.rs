//! Fonctions helper pour le calcul et l'extraction des données MACD
//!
//! Ce module contient la logique partagée pour calculer le MACD avec toutes les bougies
//! et extraire les valeurs correspondant aux bougies visibles.

use crate::finance_chart::state::ChartState;
use crate::finance_chart::core::Candle;
use super::calc::{calculate_macd, MacdValue, MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD};

/// Calcule le MACD pour toutes les bougies et retourne les valeurs MACD et les bougies visibles
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `all_macd_values` - Toutes les valeurs MACD pré-calculées
///
/// # Retourne
/// Un tuple contenant :
/// - Les valeurs MACD correspondant aux bougies visibles
/// - Les bougies visibles
/// - L'index de début des bougies visibles
pub fn calculate_macd_data<'a>(
    chart_state: &'a ChartState,
    all_macd_values: &'a [Option<MacdValue>],
) -> Option<(&'a [Option<MacdValue>], &'a [Candle], usize)> {
    if all_macd_values.is_empty() {
        return None;
    }

    // Récupérer les bougies visibles pour déterminer quelle partie du MACD afficher
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
    let end = (visible_start_idx + visible_candles_slice.len()).min(all_macd_values.len());
    let slice = &all_macd_values[visible_start_idx..end];

    Some((slice, visible_candles_slice, visible_start_idx))
}

/// Calcule toutes les valeurs MACD pour toutes les bougies
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
///
/// # Retourne
/// Toutes les valeurs MACD calculées, ou `None` si le calcul n'est pas possible
pub fn calculate_all_macd_values(chart_state: &ChartState) -> Option<Vec<Option<MacdValue>>> {
    let all_candles = chart_state.all_candles()?;
    
    if all_candles.is_empty() {
        return None;
    }

    let all_macd_values = calculate_macd(
        all_candles,
        MACD_FAST_PERIOD,
        MACD_SLOW_PERIOD,
        MACD_SIGNAL_PERIOD,
    );
    
    if all_macd_values.is_empty() {
        None
    } else {
        Some(all_macd_values)
    }
}

/// Calcule la plage de valeurs MACD (min, max) pour les valeurs visibles
///
/// # Arguments
/// * `visible_macd_values` - Les valeurs MACD visibles
///
/// # Retourne
/// `Some((min, max))` si des valeurs valides existent, `None` sinon
pub fn calculate_macd_range(
    visible_macd_values: &[Option<MacdValue>],
) -> Option<(f64, f64)> {
    let (min, max) = visible_macd_values
        .iter()
        .filter_map(|opt| opt.as_ref())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), macd| {
            let min_val = min.min(macd.macd_line).min(macd.signal_line);
            let max_val = max.max(macd.macd_line).max(macd.signal_line);
            (min_val, max_val)
        });

    if min.is_infinite() || max.is_infinite() {
        None
    } else {
        Some((min, max))
    }
}

/// Récupère la dernière valeur MACD valide
///
/// # Arguments
/// * `chart_state` - L'état du graphique (utilisé si all_macd_values est None)
/// * `all_macd_values` - Toutes les valeurs MACD pré-calculées (optionnel)
///
/// # Retourne
/// La dernière valeur MACD valide, ou `None` si aucune n'est disponible
pub fn get_last_macd_value(
    chart_state: &ChartState,
    all_macd_values: Option<&Vec<Option<MacdValue>>>,
) -> Option<MacdValue> {
    // Utiliser les valeurs pré-calculées si disponibles
    if let Some(values) = all_macd_values {
        return values.iter().rev().find_map(|opt| opt.clone());
    }
    
    // Sinon, calculer (fallback pour compatibilité)
    chart_state
        .all_candles()
        .and_then(|all_candles| {
            if all_candles.len() < MACD_SLOW_PERIOD + MACD_SIGNAL_PERIOD {
                return None;
            }
            let macd_values = calculate_macd(
                all_candles,
                MACD_FAST_PERIOD,
                MACD_SLOW_PERIOD,
                MACD_SIGNAL_PERIOD,
            );
            macd_values.iter().rev().find_map(|opt| opt.clone())
        })
}

