//! Vue de la fenêtre des actifs disponibles

use iced::widget::{column, container, row, text, Space, scrollable, button};
use iced::{Element, Length, Color};
use crate::app::app_state::ChartApp;
use crate::app::messages::Message;
use crate::app::view_styles;

/// Vue de la fenêtre des actifs disponibles
pub fn view_assets(app: &ChartApp) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("Actifs disponibles")
                .size(20)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
            Space::new().width(Length::Fill),
            button("🔄 Actualiser")
                .on_press(Message::LoadAssets)
                .style(view_styles::icon_button_style),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .padding(15),
    ]
    .spacing(10)
    .padding(20);

    if app.assets_loading {
        content = content.push(
            container(
                column![
                    text("Chargement des actifs...")
                        .size(14)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                ]
                .align_x(iced::Alignment::Center)
                .padding(40)
            )
            .width(Length::Fill)
        );
    } else if app.assets.is_empty() {
        content = content.push(
            container(
                column![
                    text("Aucun actif chargé")
                        .size(16)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    Space::new().height(Length::Fixed(10.0)),
                    text("Cliquez sur 'Actualiser' pour charger les actifs disponibles depuis le provider")
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .align_x(iced::Alignment::Center)
                .spacing(10)
                .padding(40)
            )
            .width(Length::Fill)
        );
    } else {
        // En-tête du tableau
        let header = container(
            row![
                text("Symbole")
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                    .width(Length::Fixed(130.0)),
                text("Base")
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                    .width(Length::Fixed(80.0)),
                text("Quote")
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                    .width(Length::Fixed(80.0)),
                text("Prix")
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                    .width(Length::Fixed(120.0)),
                text("Statut")
                    .size(12)
                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                    .width(Length::Fixed(90.0)),
            ]
            .spacing(10)
            .padding([10, 15])
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.18))),
            border: iced::Border {
                color: Color::from_rgb(0.2, 0.2, 0.25),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        content = content.push(header);

        // Liste des actifs dans un scrollable
        let mut assets_list = column![]
            .spacing(2);

        for symbol in &app.assets {
            let status_color = if symbol.status == "TRADING" {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.8, 0.6, 0.0)
            };

            // Formater le prix
            let price_text = if let Some(price) = symbol.price {
                if price >= 1000.0 {
                    format!("{:.2}", price)
                } else if price >= 1.0 {
                    format!("{:.4}", price)
                } else if price >= 0.0001 {
                    format!("{:.6}", price)
                } else {
                    format!("{:.8}", price)
                }
            } else {
                String::from("—")
            };
            
            let price_color = if symbol.price.is_some() {
                Color::from_rgb(0.9, 0.9, 0.9)
            } else {
                Color::from_rgb(0.5, 0.5, 0.5)
            };

            assets_list = assets_list.push(
                container(
                    row![
                        text(&symbol.symbol)
                            .size(12)
                            .color(Color::from_rgb(0.9, 0.9, 0.9))
                            .width(Length::Fixed(130.0)),
                        text(&symbol.base_asset)
                            .size(12)
                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                            .width(Length::Fixed(80.0)),
                        text(&symbol.quote_asset)
                            .size(12)
                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                            .width(Length::Fixed(80.0)),
                        text(price_text)
                            .size(12)
                            .color(price_color)
                            .width(Length::Fixed(120.0)),
                        text(&symbol.status)
                            .size(12)
                            .color(status_color)
                            .width(Length::Fixed(90.0)),
                    ]
                    .spacing(10)
                    .padding([8, 15])
                    .align_y(iced::Alignment::Center)
                )
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
                    border: iced::Border {
                        color: Color::from_rgb(0.2, 0.2, 0.25),
                        width: 0.5,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
            );
        }

        content = content.push(
            scrollable(assets_list)
                .width(Length::Fill)
                .height(Length::Fill)
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            ..Default::default()
        })
        .into()
}

