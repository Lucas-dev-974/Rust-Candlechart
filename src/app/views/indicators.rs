//! Vue de l'onglet d'indicateurs

use iced::widget::{button, checkbox, column, container, row, scrollable, stack, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::{self, colors},
    constants::INDICATORS_PANEL_WIDTH,
};
use crate::finance_chart::TOOLS_PANEL_WIDTH;

/// Définition d'un indicateur avec son nom et son état
struct Indicator {
    name: &'static str,
    is_active: bool,
    on_toggle: fn(bool) -> Message,
}

/// Vue de l'onglet d'indicateurs
pub fn indicators_panel(app: &ChartApp) -> Element<'_, Message> {
    // Liste des indicateurs disponibles avec leur état
    let indicators = vec![
        Indicator {
            name: "Volume Profile",
            is_active: app.panels.volume.visible,
            on_toggle: |_| Message::ToggleVolumePanel,
        },
        Indicator {
            name: "RSI",
            is_active: app.panels.rsi.visible,
            on_toggle: |_| Message::ToggleRSIPanel,
        },
        Indicator {
            name: "MACD",
            is_active: app.panels.macd.visible,
            on_toggle: |_| Message::ToggleMACDPanel,
        },
        Indicator {
            name: "Bollinger Bands",
            is_active: false,
            on_toggle: |_| Message::ClearPanelFocus, // TODO: implémenter
        },
        Indicator {
            name: "Moving Average",
            is_active: false,
            on_toggle: |_| Message::ClearPanelFocus, // TODO: implémenter
        },
        Indicator {
            name: "Stochastic",
            is_active: false,
            on_toggle: |_| Message::ClearPanelFocus, // TODO: implémenter
        },
    ];
    
    let mut indicators_list = column![].spacing(5);
    
    for indicator in indicators {
        let indicator_text = text(indicator.name)
            .size(13)
            .color(colors::TEXT_PRIMARY);
        
        let on_toggle_fn = indicator.on_toggle;
        let indicator_row = container(
            row![
                checkbox(indicator.is_active)
                    .on_toggle(move |checked| on_toggle_fn(checked)),
                indicator_text
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
            .padding([5, 10])
        )
        .style(move |_theme| {
            container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
                border: iced::Border {
                    color: Color::from_rgb(0.2, 0.2, 0.25),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });
        
        indicators_list = indicators_list.push(indicator_row);
    }
    
    container(
        column![
            // Header
            container(
                row![
                    text("Indicateurs")
                        .size(16)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button("✕")
                        .on_press(Message::ToolsPanel(crate::finance_chart::messages::ToolsPanelMessage::ToggleIndicatorsPanel))
                        .padding(4)
                        .style(view_styles::icon_button_style)
                ]
                .align_y(iced::Alignment::Center)
                .padding([10, 15])
            )
            .width(Length::Fill)
            .style(view_styles::header_container_style),
            // Liste des indicateurs
            container(
                scrollable(indicators_list)
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fixed(INDICATORS_PANEL_WIDTH))
    .height(Length::Fill)
    .style(view_styles::panel_container_style)
    .into()
}

/// Helper pour créer un chart avec overlay d'indicateurs si ouvert
pub fn chart_with_indicators_overlay<'a>(chart_content: Element<'a, Message>, app: &'a ChartApp) -> Element<'a, Message> {
    if app.indicators_panel_open {
        // Utiliser stack pour superposer l'overlay sur le graphique
        stack![
            // Le graphique principal (en dessous)
            chart_content,
            // L'overlay de l'onglet d'indicateurs (au-dessus, positionné en haut à gauche)
            container(
                row![
                    // Espace pour la toolbar (TOOLS_PANEL_WIDTH)
                    Space::new().width(Length::Fixed(TOOLS_PANEL_WIDTH)),
                    // L'onglet d'indicateurs
                    indicators_panel(app)
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        chart_content
    }
}

