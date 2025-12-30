//! Section "Stratégies de Trading"

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, checkbox};
use iced::{Element, Length, Color};
use crate::app::{app_state::ChartApp, messages::Message, view_styles::colors};
use std::collections::HashSet;

/// Style pour les cartes de stratégie
fn strategy_card_style(_theme: &iced::Theme) -> container::Style {
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

/// Style pour les boutons secondaires
fn secondary_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.4))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.4, 0.4, 0.5),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Style pour les boutons primaires
fn primary_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.8))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.3, 0.6, 0.9),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Style pour les boutons destructifs
fn destructive_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.7, 0.2, 0.2))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.8, 0.3, 0.3),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Liste des timeframes disponibles
const AVAILABLE_TIMEFRAMES: &[&str] = &[
    "1m", "3m", "5m", "15m", "30m",
    "1h", "2h", "4h", "6h", "8h", "12h",
    "1d", "3d",
    "1w",
    "1M",
];

/// Vue pour la section "Stratégies"
pub fn view_strategies(app: &ChartApp) -> Element<'_, Message> {
    let strategies = app.strategy_manager.get_all();
    
    let mut content = column![]
        .spacing(10)
        .padding(10);
    
    // Titre
    content = content.push(
        text("Stratégies de Trading Automatisées")
            .size(18)
            .color(colors::TEXT_PRIMARY)
    );
    
    // Liste des stratégies
    if strategies.is_empty() {
        content = content.push(
            container(
                text("Aucune stratégie enregistrée")
                    .size(14)
                    .color(colors::TEXT_SECONDARY)
            )
            .padding(20)
        );
    } else {
        for (id, reg) in strategies {
            let strategy_name = reg.strategy.name();
            let is_enabled = reg.enabled;
            let status_text = if is_enabled { "Actif" } else { "Inactif" };
            let status_color = if is_enabled {
                Color::from_rgb(0.2, 0.8, 0.3)
            } else {
                Color::from_rgb(0.6, 0.6, 0.6)
            };
            
            let id_clone = id.clone();
            let id_clone2 = id.clone();
            let id_clone3 = id.clone();
            let id_clone4 = id.clone();
            
            // Récupérer l'état d'édition
            let editing_state = app.editing_strategies.get(&id);
            let is_expanded = editing_state.map(|s| s.expanded).unwrap_or(false);
            
            // Récupérer les paramètres de la stratégie (cloner pour éviter les problèmes de borrowing)
            let parameters: Vec<_> = reg.strategy.parameters().into_iter().collect();
            
            // Récupérer les timeframes actuels
            let current_timeframes = reg.allowed_timeframes.clone().unwrap_or_default();
            let selected_timeframes: HashSet<String> = if let Some(editing) = editing_state {
                editing.selected_timeframes.iter().cloned().collect()
            } else {
                current_timeframes.iter().cloned().collect()
            };
            
            let mut strategy_card_content = column![
                row![
                    text(strategy_name)
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    text(status_text)
                        .size(12)
                        .color(status_color),
                    Space::new().width(Length::Fill),
                    button(if is_expanded { "▼ Configurer" } else { "▶ Configurer" })
                        .on_press(Message::ToggleStrategyConfig(id_clone4))
                        .style(secondary_button_style)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                text(reg.strategy.description())
                    .size(11)
                    .color(colors::TEXT_SECONDARY),
                row![
                    button(if is_enabled { "Désactiver" } else { "Activer" })
                        .on_press(if is_enabled {
                            Message::DisableStrategy(id_clone)
                        } else {
                            Message::EnableStrategy(id_clone)
                        })
                        .style(secondary_button_style),
                    button("Supprimer")
                        .on_press(Message::RemoveStrategy(id_clone2))
                        .style(destructive_button_style)
                ]
                .spacing(5)
            ]
            .spacing(8);
            
            // Panneau de configuration (si ouvert)
            if is_expanded {
                let mut config_panel = column![]
                    .spacing(10)
                    .padding(10);
                
                // Section Paramètres
                config_panel = config_panel.push(
                    text("Paramètres")
                        .size(14)
                        .color(colors::TEXT_PRIMARY)
                );
                
                for param in &parameters {
                    let param_name = param.name.clone();
                    let param_id = id.clone();
                    let param_description = param.description.clone();
                    let param_min = param.min;
                    let param_max = param.max;
                    
                    // Récupérer la valeur actuelle (depuis l'état d'édition ou la valeur réelle)
                    let current_value = if let Some(editing) = editing_state {
                        editing.param_values.get(&param_name)
                            .cloned()
                            .unwrap_or_else(|| format!("{:.2}", param.value))
                    } else {
                        format!("{:.2}", param.value)
                    };
                    
                    config_panel = config_panel.push(
                        column![
                            row![
                                text(param_description)
                                    .size(12)
                                    .color(colors::TEXT_SECONDARY),
                                Space::new().width(Length::Fill),
                                text(format!("Min: {:.2}, Max: {:.2}", param_min, param_max))
                                    .size(10)
                                    .color(colors::TEXT_SECONDARY)
                            ],
                            text_input("", &current_value)
                                .on_input(move |value| {
                                    Message::UpdateStrategyParamInput {
                                        strategy_id: param_id.clone(),
                                        param_name: param_name.clone(),
                                        value,
                                    }
                                })
                                .padding(6)
                                .size(12)
                        ]
                        .spacing(4)
                    );
                }
                
                // Section Timeframes
                config_panel = config_panel.push(
                    Space::new().height(Length::Fixed(10.0))
                );
                
                config_panel = config_panel.push(
                    text("Timeframes autorisés")
                        .size(14)
                        .color(colors::TEXT_PRIMARY)
                );
                
                config_panel = config_panel.push(
                    text("Laissez vide pour autoriser tous les timeframes")
                        .size(11)
                        .color(colors::TEXT_SECONDARY)
                );
                
                // Grille de checkboxes pour les timeframes
                let mut timeframe_rows = column![].spacing(5);
                let mut current_row = row![].spacing(10);
                let mut items_in_row = 0;
                
                for timeframe in AVAILABLE_TIMEFRAMES {
                    let tf_id = id.clone();
                    let tf_str = timeframe.to_string();
                    let is_selected = selected_timeframes.contains(&tf_str);
                    
                    current_row = current_row.push(
                        row![
                            checkbox(is_selected)
                                .on_toggle(move |_| {
                                    Message::ToggleStrategyTimeframe {
                                        strategy_id: tf_id.clone(),
                                        timeframe: tf_str.clone(),
                                    }
                                }),
                            text(*timeframe)
                                .size(12)
                                .color(colors::TEXT_SECONDARY)
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center)
                    );
                    
                    items_in_row += 1;
                    if items_in_row >= 5 {
                        timeframe_rows = timeframe_rows.push(current_row);
                        current_row = row![].spacing(10);
                        items_in_row = 0;
                    }
                }
                
                if items_in_row > 0 {
                    timeframe_rows = timeframe_rows.push(current_row);
                }
                
                config_panel = config_panel.push(timeframe_rows);
                
                // Boutons Appliquer/Annuler
                config_panel = config_panel.push(
                    Space::new().height(Length::Fixed(10.0))
                );
                
                let apply_id = id.clone();
                let cancel_id = id.clone();
                
                config_panel = config_panel.push(
                    row![
                        button("Appliquer")
                            .on_press(Message::ApplyStrategyConfig(apply_id))
                            .style(primary_button_style),
                        button("Annuler")
                            .on_press(Message::CancelStrategyConfig(cancel_id))
                            .style(secondary_button_style)
                    ]
                    .spacing(10)
                );
                
                // Container pour le panneau de configuration
                let config_container = container(config_panel)
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
                        border: iced::Border {
                            color: Color::from_rgb(0.3, 0.5, 0.7),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .padding(10);
                
                strategy_card_content = strategy_card_content.push(config_container);
            }
            
            let strategy_card = container(strategy_card_content)
                .padding(10)
                .style(strategy_card_style);
            
            content = content.push(strategy_card);
        }
    }
    
    // Boutons pour ajouter des stratégies
    content = content.push(
        Space::new().height(Length::Fixed(10.0))
    );
    
    content = content.push(
        text("Ajouter une stratégie")
            .size(14)
            .color(colors::TEXT_PRIMARY)
    );
    
    content = content.push(
        row![
            button("Stratégie RSI")
                .on_press(Message::RegisterRSIStrategy)
                .style(primary_button_style),
            button("Stratégie MA Crossover")
                .on_press(Message::RegisterMACrossoverStrategy)
                .style(primary_button_style)
        ]
        .spacing(10)
    );
    
    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}


