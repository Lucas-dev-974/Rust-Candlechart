//! Composant pour afficher les notifications à l'utilisateur
//!
//! Ce module gère l'affichage des notifications avec différents types
//! (erreur, avertissement, succès, info) et support pour l'auto-dismiss.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    state::notifications::NotificationType,
};

/// Crée la vue des notifications en overlay
pub fn notifications_overlay(app: &ChartApp) -> Element<'_, Message> {
    let notifications = app.ui.notifications.notifications();
    
    if notifications.is_empty() {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Shrink)
            .into();
    }

    let mut notification_elements = Vec::new();

    for notification in notifications.iter() {
        let (bg_color, icon) = match notification.notification_type {
            NotificationType::Error => (
                Color::from_rgb(0.9, 0.3, 0.3), // Rouge
                "❌"
            ),
            NotificationType::Warning => (
                Color::from_rgb(0.9, 0.7, 0.2), // Orange
                "⚠️"
            ),
            NotificationType::Success => (
                Color::from_rgb(0.2, 0.8, 0.4), // Vert
                "✅"
            ),
            NotificationType::Info => (
                Color::from_rgb(0.2, 0.6, 0.9), // Bleu
                "ℹ️"
            ),
        };

        let notification_element: Element<'_, Message> = container(
            row![
                text(icon)
                    .size(20),
                Space::new().width(Length::Fixed(10.0)),
                column![
                    text(&notification.message)
                        .size(14)
                        .color(Color::WHITE)
                        .width(Length::Fill),
                    if let Some(ref details) = notification.details {
                        text(details)
                            .size(11)
                            .color(Color::from_rgb(0.9, 0.9, 0.9))
                            .width(Length::Fill)
                    } else {
                        text("")
                    }
                ]
                .spacing(4)
                .width(Length::Fill),
                Space::new().width(Length::Fixed(10.0)),
                button("✕")
                    .on_press(Message::DismissNotification(notification.id))
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
            background: Some(iced::Background::Color(bg_color)),
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

        notification_elements.push(notification_element);
    }

    // Ajouter un bouton pour fermer toutes les notifications si plusieurs
    let footer = if notifications.len() > 1 {
        container(
            row![
                Space::new().width(Length::Fill),
                button("Fermer tout")
                    .on_press(Message::ClearAllNotifications)
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
                for elem in notification_elements {
                    col = col.push(elem);
                }
                col.spacing(8).width(Length::Fill)
            },
            footer
        ]
        .width(Length::Fill)
    )
    .width(Length::Fixed(360.0))
    .padding(16)
    .style(|_theme| container::Style {
        background: None,
        ..Default::default()
    })
    .into()
}
