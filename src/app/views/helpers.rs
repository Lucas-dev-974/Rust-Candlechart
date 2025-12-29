//! Helpers et utilitaires partagés entre les vues

use iced::widget::{button, container, row, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    messages::Message,
    view_styles::{self, colors},
};

/// Fonction helper pour le bouton de settings dans le coin
pub fn corner_settings_button() -> Element<'static, Message> {
    button("⚙️")
        .on_press(Message::OpenSettings)
        .padding(8)
        .style(view_styles::icon_button_style)
        .into()
}

/// Helper pour créer une vue de section de panneau simple
pub fn simple_panel_section<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    use iced::widget::column;
    
    container(
        column![
            text(title)
                .size(16)
                .color(colors::TEXT_PRIMARY),
            Space::new().height(Length::Fixed(10.0)),
            text(description)
                .size(12)
                .color(colors::TEXT_SECONDARY)
        ]
        .padding(15)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(view_styles::panel_container_no_border_style)
    .into()
}

/// Crée une ligne d'information (label + valeur)
pub fn create_info_row(label: &str, value: String, value_color: Option<Color>) -> Element<'_, Message> {
    row![
        text(label)
            .size(12)
            .color(colors::TEXT_SECONDARY),
        Space::new().width(Length::Fill),
        text(value)
            .size(12)
            .color(value_color.unwrap_or(colors::TEXT_PRIMARY))
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Crée un séparateur horizontal
pub fn separator() -> Element<'static, Message> {
    container(Space::new().height(1))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.35))),
            ..Default::default()
        })
        .into()
}

/// Crée une barre de progression indéterminée (animation de chargement)
pub fn progress_bar() -> Element<'static, Message> {
    // Utiliser un container avec un fond coloré simple
    container(Space::new())
        .width(Length::Fixed(200.0))
        .height(Length::Fixed(4.0))
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.6, 1.0))),
            ..Default::default()
        })
        .into()
}


