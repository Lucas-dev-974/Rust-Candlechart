//! Vue de la section Compte

use iced::widget::{checkbox, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::colors,
};
use super::helpers::create_info_row;

/// Style de carte pour les sections
fn section_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Crée la section Solde et Équité
fn create_balance_section(app: &ChartApp) -> Element<'_, Message> {
    let info = &app.account_info;
    
    container(
        column![
            text("Solde et Équité")
                .size(14)
                .color(colors::TEXT_PRIMARY),
            Space::new().height(Length::Fixed(12.0)),
            create_info_row("Solde total", format!("{:.2} USDT", info.total_balance), None),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("Équité", format!("{:.2} USDT", info.equity), {
                let pnl = info.unrealized_pnl;
                Some(if pnl >= 0.0 {
                    Color::from_rgb(0.0, 0.8, 0.0)
                } else {
                    Color::from_rgb(0.8, 0.0, 0.0)
                })
            }),
        ]
        .padding(12)
        .spacing(8)
    )
    .style(section_card_style)
    .into()
}

/// Crée la section Marge
fn create_margin_section(app: &ChartApp) -> Element<'_, Message> {
    let info = &app.account_info;
    
    let margin_level_color = if info.liquidation {
        Color::from_rgb(0.8, 0.0, 0.0)
    } else if info.margin_call {
        Color::from_rgb(1.0, 0.6, 0.0)
    } else if info.margin_level > 200.0 {
        Color::from_rgb(0.0, 0.8, 0.0)
    } else if info.margin_level > 100.0 {
        Color::from_rgb(0.8, 0.8, 0.0)
    } else {
        Color::from_rgb(0.8, 0.0, 0.0)
    };
    
    container(
        column![
            text("Marge")
                .size(14)
                .color(colors::TEXT_PRIMARY),
            Space::new().height(Length::Fixed(12.0)),
            create_info_row("Marge utilisée", format!("{:.2} USDT", info.used_margin), None),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("Marge libre", format!("{:.2} USDT", info.free_margin), None),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("Niveau de marge", format!("{:.2}%", info.margin_level), Some(margin_level_color)),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("Effet de levier", format!("{}x", info.leverage), None),
        ]
        .padding(12)
        .spacing(8)
    )
    .style(section_card_style)
    .into()
}

/// Crée la section P&L
fn create_pnl_section(app: &ChartApp) -> Element<'_, Message> {
    let info = &app.account_info;
    let total_pnl = info.unrealized_pnl + info.realized_pnl;
    
    container(
        column![
            text("Profit & Loss")
                .size(14)
                .color(colors::TEXT_PRIMARY),
            Space::new().height(Length::Fixed(12.0)),
            create_info_row("P&L non réalisé", format!("{:.2} USDT", info.unrealized_pnl), Some(if info.unrealized_pnl >= 0.0 { Color::from_rgb(0.0, 0.8, 0.0) } else { Color::from_rgb(0.8, 0.0, 0.0) })),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("P&L réalisé", format!("{:.2} USDT", info.realized_pnl), Some(if info.realized_pnl >= 0.0 { Color::from_rgb(0.0, 0.8, 0.0) } else { Color::from_rgb(0.8, 0.0, 0.0) })),
            Space::new().height(Length::Fixed(8.0)),
            create_info_row("P&L total", format!("{:.2} USDT", total_pnl), Some(if total_pnl >= 0.0 { Color::from_rgb(0.0, 0.8, 0.0) } else { Color::from_rgb(0.8, 0.0, 0.0) })),
        ]
        .padding(12)
        .spacing(8)
    )
    .style(section_card_style)
    .into()
}

/// Crée la section Positions et Risque
fn create_positions_section(app: &ChartApp) -> Element<'_, Message> {
    let info = &app.account_info;
    
    container(
        column![
            text("Positions et Risque")
                .size(14)
                .color(colors::TEXT_PRIMARY),
            Space::new().height(Length::Fixed(12.0)),
            create_info_row("Positions ouvertes", format!("{}", info.open_positions), None),
        ]
        .padding(12)
        .spacing(8)
    )
    .style(section_card_style)
    .into()
}

/// Vue pour la section "Compte"
pub fn view_bottom_panel_account(app: &ChartApp) -> Element<'_, Message> {
    let is_demo_mode = app.account_type.is_demo();
    let is_real_mode = app.account_type.is_real();
    
    // Informations sur le provider actif
    let provider_name = app.provider_config.active_provider.display_name();
    
    // Label à afficher : "Démo" en mode paper, sinon le nom du provider
    let provider_label = if is_demo_mode {
        String::from("Démo")
    } else {
        provider_name.to_string()
    };
    
    // Déterminer le statut de connexion pour la pastille
    let is_connected = if is_real_mode {
        app.provider_connection_status.unwrap_or(false)
    } else {
        true // En mode paper, considéré comme "connecté"
    };
    
    // Couleur de la pastille selon le statut
    let status_color = if is_connected {
        Color::from_rgb(0.0, 0.8, 0.0)
    } else {
        Color::from_rgb(0.8, 0.0, 0.0)
    };
    
    // Layout principal
    container(
        scrollable(
            column![
                // Header avec titre, provider avec pastille, et switch
                row![
                    // Titre avec provider et pastille de statut
                    row![
                        text("Compte")
                            .size(20)
                            .color(colors::TEXT_PRIMARY),
                        Space::new().width(Length::Fixed(10.0)),
                        text(format!("• {}", provider_label))
                            .size(14)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fixed(8.0)),
                        // Pastille de statut de connexion
                        container(Space::new())
                            .width(Length::Fixed(10.0))
                            .height(Length::Fixed(10.0))
                            .style(move |_theme| {
                                container::Style {
                                    background: Some(iced::Background::Color(status_color)),
                                    border: iced::Border {
                                        color: status_color,
                                        width: 0.0,
                                        radius: 5.0.into(),
                                    },
                                    ..Default::default()
                                }
                            }),
                    ]
                    .align_y(iced::Alignment::Center),
                    Space::new().width(Length::Fill),
                    // Switch pour basculer entre démo et réel
                    row![
                        checkbox(is_demo_mode)
                            .on_toggle(move |_| Message::ToggleAccountType),
                        text("Mode Paper Trading")
                            .size(13)
                            .color(colors::TEXT_SECONDARY),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                ]
                .align_y(iced::Alignment::Center)
                .width(Length::Fill),
                
                Space::new().height(Length::Fixed(20.0)),
                create_balance_section(app),
                Space::new().height(Length::Fixed(20.0)),
                create_margin_section(app),
                Space::new().height(Length::Fixed(20.0)),
                create_pnl_section(app),
                Space::new().height(Length::Fixed(20.0)),
                create_positions_section(app),
            ]
            .spacing(0)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
        ..Default::default()
    })
    .into()
}

