//! Fonctions helper pour le calcul et l'extraction des données MACD
//!
//! Ce module contient la logique partagée pour calculer le MACD avec toutes les bougies
//! et extraire les valeurs correspondant aux bougies visibles.

use super::state::ChartState;
use super::core::Candle;
use super::indicators::{calculate_macd, MacdValue, MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD};

/// Calcule le MACD pour toutes les bougies et retourne les valeurs MACD et les bougies visibles
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
///
/// # Retourne
/// Un tuple contenant :
/// - Toutes les valeurs MACD calculées (doit être gardé en vie)
/// - Les valeurs MACD correspondant aux bougies visibles (références)
/// - Les bougies visibles
/// - L'index de début des bougies visibles
///
/// # Note
/// Les références dans le tuple pointent vers `all_macd_values` qui doit rester en vie
/// pendant l'utilisation des références.
pub fn calculate_macd_data<'a>(
    chart_state: &'a ChartState,
    all_macd_values: &'a Vec<Option<MacdValue>>,
) -> Option<(
    Vec<&'a Option<MacdValue>>,
    &'a [Candle],
    usize,
)> {
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

    // Extraire uniquement les valeurs MACD correspondant aux bougies visibles
    let visible_macd_values: Vec<_> = all_macd_values
        .iter()
        .skip(visible_start_idx)
        .take(visible_candles_slice.len())
        .collect();

    Some((
        visible_macd_values,
        visible_candles_slice,
        visible_start_idx,
    ))
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
pub fn calculate_macd_range<'a>(
    visible_macd_values: &[&'a Option<MacdValue>],
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
/// * `all_macd_values` - Toutes les valeurs MACD pré-calculées (optionnel)
/// * `chart_state` - L'état du graphique (utilisé si all_macd_values est None)
///
/// # Retourne
/// La dernière valeur MACD valide, ou `None` si aucune n'est disponible
///
/// # Note
/// Si `all_macd_values` est fourni, cette fonction évite de recalculer toutes les valeurs MACD.
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
            // Prendre la dernière valeur MACD valide
            macd_values.iter().rev().find_map(|opt| opt.clone())
        })
}

