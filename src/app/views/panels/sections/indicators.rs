//! Section "Indicateurs"

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, pick_list};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::colors,
};
use super::super::super::helpers::simple_panel_section;

/// Vue pour la section "Indicateurs"
pub fn view_indicators(app: &ChartApp) -> Element<'_, Message> {
    // Section "Indicateurs actifs" - tous les indicateurs actifs (panneaux et overlay)
    let mut active_indicators = column![].spacing(4);
    
    // Vérifier tous les indicateurs actifs
    let has_active_indicators = app.ui.panels.volume.visible
        || app.ui.panels.rsi.visible
        || app.ui.panels.macd.visible
        || app.indicators.bollinger_bands_enabled
        || app.indicators.moving_average_enabled;
    
    if has_active_indicators {
        // Titre de la section
        active_indicators = active_indicators.push(
            text("Indicateurs actifs")
                .size(14)
                .color(colors::TEXT_SECONDARY)
        );
        
        // Volume Profile
        if app.ui.panels.volume.visible {
            let indicator_row = container(
                row![
                    text("Volume Profile")
                        .size(12)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button(
                        text("×")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let background = match status {
                            iced::widget::button::Status::Pressed => colors::DANGER,
                            iced::widget::button::Status::Hovered => Color::from_rgb(0.6, 0.2, 0.2),
                            _ => Color::from_rgb(0.4, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(background)),
                            border: iced::Border {
                                color: colors::BORDER_STANDARD,
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            text_color: colors::TEXT_PRIMARY,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::ToggleVolumePanel)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .padding([6, 10])
            )
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
                    border: iced::Border {
                        color: colors::BORDER_STANDARD,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            });
            
            active_indicators = active_indicators.push(indicator_row);
        }
        
        // RSI
        if app.ui.panels.rsi.visible {
            let rsi_period_str = app.indicators.params.rsi_period.to_string();
            let indicator_content = column![
                // Header avec nom et bouton supprimer
                row![
                    text("RSI")
                        .size(12)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button(
                        text("×")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let background = match status {
                            iced::widget::button::Status::Pressed => colors::DANGER,
                            iced::widget::button::Status::Hovered => Color::from_rgb(0.6, 0.2, 0.2),
                            _ => Color::from_rgb(0.4, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(background)),
                            border: iced::Border {
                                color: colors::BORDER_STANDARD,
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            text_color: colors::TEXT_PRIMARY,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::ToggleRSIPanel)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                // Paramètres
                column![
                    row![
                        text("Période:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("14", &rsi_period_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateRSIPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Méthode:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        pick_list(
                            vec!["Wilder".to_string(), "Simple".to_string()],
                            Some(app.indicators.params.rsi_method.as_str().to_string()),
                            move |selected: String| {
                                if let Some(method) = crate::app::state::RSIMethod::from_str(&selected) {
                                    Message::UpdateRSIMethod(method)
                                } else {
                                    Message::ClearPanelFocus
                                }
                            }
                        )
                        .width(Length::Fixed(100.0))
                        .text_size(11.0)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ]
                .spacing(4)
                .padding([8.0, 10.0])
            ]
            .spacing(4);
            
            let indicator_row = container(indicator_content)
                .padding([6, 10])
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
                        border: iced::Border {
                            color: colors::BORDER_STANDARD,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                });
            
            active_indicators = active_indicators.push(indicator_row);
        }
        
        // MACD
        if app.ui.panels.macd.visible {
            let macd_fast_str = app.indicators.params.macd_fast_period.to_string();
            let macd_slow_str = app.indicators.params.macd_slow_period.to_string();
            let macd_signal_str = app.indicators.params.macd_signal_period.to_string();
            let indicator_content = column![
                // Header avec nom et bouton supprimer
                row![
                    text("MACD")
                        .size(12)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button(
                        text("×")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let background = match status {
                            iced::widget::button::Status::Pressed => colors::DANGER,
                            iced::widget::button::Status::Hovered => Color::from_rgb(0.6, 0.2, 0.2),
                            _ => Color::from_rgb(0.4, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(background)),
                            border: iced::Border {
                                color: colors::BORDER_STANDARD,
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            text_color: colors::TEXT_PRIMARY,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::ToggleMACDPanel)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                // Paramètres
                column![
                    row![
                        text("EMA Rapide:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("12", &macd_fast_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateMACDFastPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("EMA Lente:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("26", &macd_slow_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateMACDSlowPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Signal:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("9", &macd_signal_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateMACDSignalPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ]
                .spacing(4)
                .padding([8.0, 10.0])
            ]
            .spacing(4);
            
            let indicator_row = container(indicator_content)
                .padding([6, 10])
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
                        border: iced::Border {
                            color: colors::BORDER_STANDARD,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                });
            
            active_indicators = active_indicators.push(indicator_row);
        }
        
        // Bollinger Bands
        if app.indicators.bollinger_bands_enabled {
            let bb_period_str = app.indicators.params.bollinger_period.to_string();
            let bb_std_dev_str = format!("{:.1}", app.indicators.params.bollinger_std_dev);
            let indicator_content = column![
                // Header avec nom et bouton supprimer
                row![
                    text("Bollinger Bands")
                        .size(12)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button(
                        text("×")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let background = match status {
                            iced::widget::button::Status::Pressed => colors::DANGER,
                            iced::widget::button::Status::Hovered => Color::from_rgb(0.6, 0.2, 0.2),
                            _ => Color::from_rgb(0.4, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(background)),
                            border: iced::Border {
                                color: colors::BORDER_STANDARD,
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            text_color: colors::TEXT_PRIMARY,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::ToggleBollingerBands)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                // Paramètres
                column![
                    row![
                        text("Période:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("20", &bb_period_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateBollingerPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Écart-type:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("2.0", &bb_std_dev_str)
                            .on_input(|s| {
                                s.parse::<f64>()
                                    .ok()
                                    .filter(|&v| v > 0.0 && v <= 10.0)
                                    .map(Message::UpdateBollingerStdDev)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ]
                .spacing(4)
                .padding([8.0, 10.0])
            ]
            .spacing(4);
            
            let indicator_row = container(indicator_content)
                .padding([6, 10])
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
                        border: iced::Border {
                            color: colors::BORDER_STANDARD,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                });
            
            active_indicators = active_indicators.push(indicator_row);
        }
        
        // Moving Average
        if app.indicators.moving_average_enabled {
            let ma_period_str = app.indicators.params.ma_period.to_string();
            let indicator_content = column![
                // Header avec nom et bouton supprimer
                row![
                    text("Moving Average")
                        .size(12)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    button(
                        text("×")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let background = match status {
                            iced::widget::button::Status::Pressed => colors::DANGER,
                            iced::widget::button::Status::Hovered => Color::from_rgb(0.6, 0.2, 0.2),
                            _ => Color::from_rgb(0.4, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(background)),
                            border: iced::Border {
                                color: colors::BORDER_STANDARD,
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            text_color: colors::TEXT_PRIMARY,
                            ..Default::default()
                        }
                    })
                    .on_press(Message::ToggleMovingAverage)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                // Paramètres
                column![
                    row![
                        text("Période:")
                            .size(11)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text_input("20", &ma_period_str)
                            .on_input(|s| {
                                s.parse::<usize>()
                                    .ok()
                                    .filter(|&v| v > 0 && v <= 200)
                                    .map(Message::UpdateMAPeriod)
                                    .unwrap_or(Message::ClearPanelFocus)
                            })
                            .padding(4)
                            .width(Length::Fixed(60.0))
                            .size(11)
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                ]
                .spacing(4)
                .padding([8.0, 10.0])
            ]
            .spacing(4);
            
            let indicator_row = container(indicator_content)
                .padding([6, 10])
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
                        border: iced::Border {
                            color: colors::BORDER_STANDARD,
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                });
            
            active_indicators = active_indicators.push(indicator_row);
        }
    }
    
    // Contenu de la section
    let content = if has_active_indicators {
        column![
            container(
                column![
                    active_indicators
                ]
                .spacing(8)
            )
            .padding(15)
            .width(Length::Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.08, 0.08, 0.10, 0.95))),
                    border: iced::Border {
                        color: colors::BORDER_STANDARD,
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            })
        ]
        .spacing(10)
        .padding(10)
    } else {
        column![
            simple_panel_section(
                "Indicateurs techniques",
                "Aucun indicateur actif. Activez des indicateurs depuis le panneau d'indicateurs."
            )
        ]
    };
    
    container(
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}



