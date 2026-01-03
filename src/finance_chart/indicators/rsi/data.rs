//! Fonctions helper pour le calcul et l'extraction des données RSI
//!
//! Ce module contient la logique partagée pour calculer le RSI avec toutes les bougies
//! et extraire les valeurs correspondant aux bougies visibles.

use crate::finance_chart::state::ChartState;
use crate::finance_chart::core::Candle;
use crate::app::state::{RSIMethod, IndicatorParams};
use super::calc::{calculate_rsi, RSI_PERIOD};

/// Calcule le RSI pour toutes les bougies et retourne les valeurs RSI et les bougies visibles
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `all_rsi_values` - Toutes les valeurs RSI pré-calculées
///
/// # Retourne
/// Un tuple contenant :
/// - Les valeurs RSI correspondant aux bougies visibles (références)
/// - Les bougies visibles
/// - L'index de début des bougies visibles
pub fn calculate_rsi_data<'a>(
    chart_state: &'a ChartState,
    all_rsi_values: &'a Vec<Option<f64>>,
) -> Option<(
    Vec<&'a Option<f64>>,
    &'a [Candle],
    usize,
)> {
    if all_rsi_values.is_empty() {
        return None;
    }

    // Récupérer les bougies visibles pour déterminer quelle partie du RSI afficher
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

    // Extraire uniquement les valeurs RSI correspondant aux bougies visibles
    let visible_rsi_values: Vec<_> = all_rsi_values
        .iter()
        .skip(visible_start_idx)
        .take(visible_candles_slice.len())
        .collect();

    Some((
        visible_rsi_values,
        visible_candles_slice,
        visible_start_idx,
    ))
}

/// Calcule toutes les valeurs RSI pour toutes les bougies
///
/// # Arguments
/// * `chart_state` - L'état du graphique contenant les bougies
/// * `params` - Paramètres de l'indicateur RSI (période et méthode)
///
/// # Retourne
/// Toutes les valeurs RSI calculées, ou `None` si le calcul n'est pas possible
pub fn calculate_all_rsi_values(chart_state: &ChartState, params: &IndicatorParams) -> Option<Vec<Option<f64>>> {
    let all_candles = chart_state.all_candles()?;
    
    if all_candles.is_empty() {
        return None;
    }

    let all_rsi_values = calculate_rsi(
        all_candles,
        params.rsi_period,
        params.rsi_method,
    );
    
    if all_rsi_values.is_empty() {
        None
    } else {
        Some(all_rsi_values)
    }
}

/// Récupère la dernière valeur RSI valide
///
/// # Arguments
/// * `chart_state` - L'état du graphique (utilisé si all_rsi_values est None)
/// * `all_rsi_values` - Toutes les valeurs RSI pré-calculées (optionnel)
/// * `params` - Paramètres de l'indicateur RSI (utilisé si all_rsi_values est None)
///
/// # Retourne
/// La dernière valeur RSI valide, ou `None` si aucune n'est disponible
pub fn get_last_rsi_value(
    chart_state: &ChartState,
    all_rsi_values: Option<&Vec<Option<f64>>>,
    params: Option<&IndicatorParams>,
) -> Option<f64> {
    // Utiliser les valeurs pré-calculées si disponibles
    if let Some(values) = all_rsi_values {
        return values.iter().rev().find_map(|opt| *opt);
    }
    
    // Sinon, calculer (fallback pour compatibilité)
    if let Some(params) = params {
        chart_state
            .all_candles()
            .and_then(|all_candles| {
                if all_candles.len() < params.rsi_period + 1 {
                    return None;
                }
                let rsi_values = calculate_rsi(all_candles, params.rsi_period, params.rsi_method);
                // Prendre la dernière valeur RSI valide
                rsi_values.iter().rev().find_map(|opt| *opt)
            })
    } else {
        // Fallback avec valeurs par défaut si params n'est pas fourni
        chart_state
            .all_candles()
            .and_then(|all_candles| {
                if all_candles.len() < RSI_PERIOD + 1 {
                    return None;
                }
                let rsi_values = calculate_rsi(all_candles, RSI_PERIOD, RSIMethod::default());
                // Prendre la dernière valeur RSI valide
                rsi_values.iter().rev().find_map(|opt| *opt)
            })
    }
}

