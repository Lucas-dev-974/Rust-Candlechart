//! Select box pour choisir les séries à afficher

use iced::widget::{container, pick_list, row, text, Space};
use iced::{Color, Element, Length};
use std::collections::HashSet;

use super::core::SeriesManager;
use super::messages::SeriesPanelMessage;
use crate::app::utils::utils::interval_to_seconds;

/// Liste des unités de temps valides (triées du plus petit au plus grand)
const VALID_TIME_UNITS: &[&str] = &[
    "1m", "3m", "5m", "15m", "30m",
    "1h", "2h", "4h", "6h", "8h", "12h",
    "1d", "3d",
    "1w",
    "1M",
];

/// Vérifie si un intervalle est une unité de temps valide
fn is_valid_time_unit(interval: &str) -> bool {
    VALID_TIME_UNITS.contains(&interval)
}

/// Crée un élément select box pour les séries
pub fn series_select_box<'a>(
    series_manager: &'a SeriesManager,
    selected_asset_symbol: Option<&'a String>,
) -> Element<'a, SeriesPanelMessage> {
    // Extraire les intervalles uniques de toutes les séries et filtrer pour ne garder que les unités de temps
    let mut intervals: Vec<String> = series_manager
        .all_series()
        .map(|series| series.interval.clone())
        .filter(|interval| is_valid_time_unit(interval))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    // Trier les intervalles par durée (du plus petit au plus grand)
    intervals.sort_by(|a, b| {
        let seconds_a = interval_to_seconds(a);
        let seconds_b = interval_to_seconds(b);
        seconds_a.cmp(&seconds_b)
    });

    // Trouver l'intervalle de la première série active (ou None)
    let selected = series_manager
        .active_series()
        .next()
        .map(|series| series.interval.clone());

    // Utiliser le symbole mémorisé depuis le pick_list, ou le symbole de la série active comme fallback
    let preferred_symbol = selected_asset_symbol
        .cloned()
        .or_else(|| {
            series_manager
                .active_series()
                .next()
                .map(|series| series.symbol.clone())
        });

    // Créer une correspondance intervalle -> nom de série
    // Prioriser les séries avec le symbole préféré (mémorisé ou actif), sinon prendre la première trouvée
    let interval_to_series: Vec<(String, String)> = intervals
        .iter()
        .map(|interval| {
            let series_name = series_manager
                .all_series()
                .find(|series| {
                    series.interval == *interval
                        && preferred_symbol.as_ref().map_or(true, |sym| series.symbol == *sym)
                })
                .or_else(|| {
                    series_manager
                        .all_series()
                        .find(|series| series.interval == *interval)
                })
                .map(|series| series.full_name())
                .unwrap_or_else(|| {
                    // Fallback: construire le nom avec le symbole préféré ou le premier symbole trouvé
                    if let Some(ref symbol) = preferred_symbol {
                        format!("{}_{}", symbol, interval)
                    } else {
                        series_manager
                            .all_series()
                            .find(|series| series.interval == *interval)
                            .map(|series| series.full_name())
                            .unwrap_or_default()
                    }
                });
            (interval.clone(), series_name)
        })
        .collect();

    // Label
    let label = text("Série:")
        .size(14)
        .color(Color::from_rgb(0.8, 0.8, 0.8));

    // Pick list - trouver le SeriesId correspondant à l'intervalle sélectionné
    let pick_list_widget = pick_list(
        intervals.clone(),
        selected,
        move |selected_interval: String| {
            // Trouver le nom de série correspondant à cet intervalle
            let series_name = interval_to_series
                .iter()
                .find(|(interval, _)| *interval == selected_interval)
                .map(|(_, name)| name.clone())
                .unwrap_or_default();
            
            SeriesPanelMessage::SelectSeriesByName { series_name }
        },
    )
    .width(Length::Fixed(180.0))
    .placeholder("Sélectionner une série...")
    .text_size(13.0);

    // Container avec le label et le pick_list
    container(
        row![
            label,
            Space::new().width(Length::Fixed(8.0)),
            pick_list_widget
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center)
    )
    .padding([5, 10])
    .into()
}

