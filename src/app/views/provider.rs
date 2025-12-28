//! Vue de la configuration des providers

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Color};
use crate::finance_chart::ProviderType;
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::{self, colors},
};

/// Vue de la configuration des providers
pub fn view_provider_config(app: &ChartApp) -> Element<'_, Message> {
    let title = text("Configuration des Providers")
        .size(20)
        .color(colors::TEXT_PRIMARY);

    let mut provider_list = column![].spacing(15);

    for provider_type in ProviderType::all() {
        let is_active = app.provider_config.active_provider == provider_type;
        let provider_name = text(provider_type.display_name())
            .size(16)
            .color(if is_active { colors::INFO } else { colors::TEXT_PRIMARY });
        
        let description = text(provider_type.description())
            .size(12)
            .color(colors::TEXT_SECONDARY);

        // Token input
        let current_token = app.editing_provider_token
            .get(&provider_type)
            .cloned()
            .unwrap_or_default();
        
        // Récupérer le token actuel
        let actual_token = if current_token.is_empty() {
            app.provider_config
                .providers
                .get(&provider_type)
                .and_then(|c| c.api_token.clone())
                .unwrap_or_default()
        } else {
            current_token.clone()
        };
        
        let has_token = !actual_token.is_empty();
        
        // Déterminer l'état de connexion pour le provider actif
        let (connection_status_text, is_connected) = if is_active {
            if let Some(connection_status) = app.provider_connection_status {
                if connection_status {
                    (String::from("Connecté"), true)
                } else {
                    (String::from("Non connecté"), false)
                }
            } else if app.provider_connection_testing {
                (String::from("Test en cours..."), false)
            } else if has_token {
                (String::from("Non testé"), false)
            } else {
                (String::from("Non connecté"), false)
            }
        } else {
            (String::from(""), false)
        };
        
        let token_input = text_input("API Token (optionnel)", &current_token)
            .on_input(move |token| Message::UpdateProviderToken(provider_type, token))
            .padding(8);

        // Bouton de sélection
        let select_btn = if is_active {
            button(text("✓ Actif").size(12))
                .style(view_styles::success_button_style)
        } else {
            button(text("Sélectionner").size(12))
                .on_press(Message::SelectProvider(provider_type))
                .style(view_styles::icon_button_style)
        };

        let mut provider_card_content = column![
            row![
                provider_name,
                Space::new().width(Length::Fill),
                select_btn
            ]
            .align_y(iced::Alignment::Center)
            .spacing(10),
            description,
            Space::new().height(Length::Fixed(5.0)),
            token_input,
        ]
        .spacing(8);
        
        // Ajouter le statut de connexion et le bouton de test pour le provider actif
        if is_active {
            // Badge de statut de connexion
            let connection_badge: Element<'_, Message> = container(
                text(connection_status_text.clone())
                    .size(11)
                    .color(if is_connected {
                        Color::from_rgb(0.7, 0.9, 0.7)
                    } else {
                        Color::from_rgb(1.0, 0.7, 0.7)
                    })
            )
            .padding([3, 8])
            .style(move |_theme: &iced::Theme| {
                let connected: bool = is_connected;
                container::Style {
                    background: Some(iced::Background::Color(if connected {
                        Color::from_rgb(0.15, 0.3, 0.15)
                    } else {
                        Color::from_rgb(0.3, 0.15, 0.15)
                    })),
                    border: iced::Border {
                        color: if connected {
                            Color::from_rgb(0.4, 0.8, 0.4)
                        } else {
                            Color::from_rgb(0.8, 0.4, 0.4)
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into();
            
            provider_card_content = provider_card_content
                .push(Space::new().height(Length::Fixed(10.0)))
                .push(
                    row![
                        text("Statut de connexion:")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        connection_badge
                    ]
                    .align_y(iced::Alignment::Center)
                )
                .push(Space::new().height(Length::Fixed(8.0)))
                .push(
                    button(
                        text(if app.provider_connection_testing {
                            "Test en cours..."
                        } else {
                            "Tester la connexion"
                        })
                        .size(12)
                    )
                    .on_press(Message::TestProviderConnection)
                    .padding([6, 12])
                    .style(if app.provider_connection_testing {
                        view_styles::icon_button_style
                    } else {
                        view_styles::success_button_style
                    })
                );
        }

        let provider_card = container(provider_card_content)
            .padding(15)
            .style(view_styles::provider_card_style(is_active));

        provider_list = provider_list.push(provider_card);
    }

    let apply_btn = button(
        text("Appliquer").size(14)
    )
    .on_press(Message::ApplyProviderConfig)
    .padding([8, 20])
    .style(view_styles::success_button_style);

    let cancel_btn = button(
        text("Annuler").size(14)
    )
    .on_press(Message::CancelProviderConfig)
    .padding([8, 20])
    .style(view_styles::danger_button_style);

    let content = column![
        title,
        Space::new().height(Length::Fixed(20.0)),
        scrollable(provider_list)
            .width(Length::Fill)
            .height(Length::Fill),
        Space::new().height(Length::Fixed(15.0)),
        row![
            cancel_btn,
            Space::new().width(Length::Fill),
            apply_btn
        ]
        .spacing(10)
    ]
    .spacing(15)
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(view_styles::dark_background_style)
        .into()
}

