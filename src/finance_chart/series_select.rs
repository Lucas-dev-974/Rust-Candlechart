//! Select box pour choisir les séries à afficher

use iced::widget::{container, pick_list, row, text, Space};
use iced::{Color, Element, Length};

use super::core::SeriesManager;
use super::messages::SeriesPanelMessage;

/// Crée un élément select box pour les séries
pub fn series_select_box<'a>(
    series_manager: &'a SeriesManager,
) -> Element<'a, SeriesPanelMessage> {
    // Créer la liste des options (noms des séries)
    let options: Vec<String> = series_manager
        .all_series()
        .map(|series| series.full_name())
        .collect();

    // Trouver le nom de la première série active (ou None)
    let selected = series_manager
        .active_series()
        .next()
        .map(|series| series.full_name());

    // Label
    let label = text("Série:")
        .size(14)
        .color(Color::from_rgb(0.8, 0.8, 0.8));

    // Pick list - trouver le SeriesId correspondant au nom sélectionné
    let pick_list_widget = pick_list(
        options,
        selected,
        move |series_name: String| {
            // Trouver le SeriesId correspondant au nom
            // On doit créer un SeriesId depuis le nom
            // Le nom est au format "SYMBOL_INTERVAL", donc on peut le reconstruire
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

