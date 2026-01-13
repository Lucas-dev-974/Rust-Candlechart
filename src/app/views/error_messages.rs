//! Composant pour afficher les messages d'erreur à l'utilisateur

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    error_handling::ErrorType,
};

/// Crée la vue des messages d'erreur en overlay
pub fn error_messages_overlay(app: &ChartApp) -> Element<'_, Message> {
    if app.ui.error_messages.is_empty() {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Shrink)
            .into();
    }

    let mut error_elements = Vec::new();

    for (index, error) in app.ui.error_messages.iter().enumerate() {
        let error_color = match error.error_type {
            ErrorType::Network | ErrorType::Api => Color::from_rgb(0.9, 0.3, 0.3), // Rouge
            ErrorType::Validation => Color::from_rgb(0.9, 0.7, 0.2), // Orange
            ErrorType::Parse => Color::from_rgb(0.7, 0.7, 0.2), // Jaune
            ErrorType::Configuration => Color::from_rgb(0.9, 0.3, 0.3), // Rouge
            ErrorType::Unknown => Color::from_rgb(0.7, 0.7, 0.7), // Gris
        };

        let error_icon = match error.error_type {
            ErrorType::Network => "🌐",
            ErrorType::Api => "⚠️",
            ErrorType::Validation => "✏️",
            ErrorType::Parse => "📄",
            ErrorType::Configuration => "⚙️",
            ErrorType::Unknown => "❓",
        };

        let error_element: Element<'_, Message> = container(
            row![
                text(error_icon)
                    .size(20),
                Space::new().width(Length::Fixed(10.0)),
                column![
                    text(&error.user_message)
                        .size(14)
                        .color(Color::WHITE)
                        .width(Length::Fill),
                    if error.technical_message != error.user_message {
                        text(&error.technical_message)
                            .size(11)
                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                            .width(Length::Fill)
                    } else {
                        text("")
                    }
                ]
                .spacing(4)
                .width(Length::Fill),
                Space::new().width(Length::Fixed(10.0)),
                button("✕")
                    .on_press(Message::DismissError(index))
                    .style(|_theme: &iced::Theme, _status| button::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.5))),
                        text_color: Color::WHITE,
                        border: iced::Border {
                            width: 1.0,
                            radius: 4.0.into(),
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    })
                    .padding(4)
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
            .padding(12)
        )
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(error_color)),
            border: iced::Border {
                width: 1.0,
                radius: 8.0.into(),
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .padding(8)
        .into();

        error_elements.push(error_element);
    }

    // Ajouter un bouton pour fermer toutes les erreurs si plusieurs
    let footer = if app.ui.error_messages.len() > 1 {
        container(
            row![
                Space::new().width(Length::Fill),
                button("Fermer tout")
                    .on_press(Message::ClearAllErrors)
                    .style(|_theme: &iced::Theme, _status| button::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.4, 0.4, 0.4))),
                        text_color: Color::WHITE,
                        border: iced::Border {
                            width: 1.0,
                            radius: 4.0.into(),
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    })
                    .padding(6)
            ]
            .width(Length::Fill)
        )
        .padding([8.0, 8.0])
    } else {
        container(Space::new())
    };

    container(
        column![
            {
                let mut col = column![];
                for elem in error_elements {
                    col = col.push(elem);
                }
                col.spacing(8).width(Length::Fill)
            },
            footer
        ]
        .width(Length::Fill)
    )
    .width(Length::Fill)
    .padding(16)
    .style(|_theme| container::Style {
        background: None,
        ..Default::default()
    })
    .into()
}
