//! Vues de l'application Iced
//!
//! Ce module contient toutes les méthodes de rendu (view) pour les différentes fenêtres
//! de l'application : fenêtre principale, settings, et configuration des providers.

use iced::widget::{button, column, container, row, text, scrollable, Space, checkbox, text_input};
use iced::{Element, Length, Color};
use crate::finance_chart::{
    chart, x_axis, y_axis, tools_panel, series_select_box,
    volume_chart, volume_y_axis,
    X_AXIS_HEIGHT, TOOLS_PANEL_WIDTH,
    settings::{color_fields, preset_colors, SerializableColor},
    ProviderType, VolumeScale,
};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    resize_handle::{horizontal_resize_handle, vertical_resize_handle},
    constants::VOLUME_CHART_HEIGHT,
};

/// Fonction helper pour le bouton de settings dans le coin
fn corner_settings_button() -> Element<'static, Message> {
    button("⚙️")
        .on_press(Message::OpenSettings)
        .padding(8)
        .style(|_theme, status| {
            let bg_color = match status {
                button::Status::Hovered => Color::from_rgb(0.2, 0.2, 0.25),
                _ => Color::from_rgb(0.15, 0.15, 0.18),
            };
            button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: Color::WHITE,
                ..Default::default()
            }
        })
        .into()
}

/// Composant qui regroupe toutes les sections du graphique
/// (tools_panel, chart, y_axis, x_axis, volume_chart, volume_y_axis)
fn view_chart_component(app: &ChartApp) -> Element<'_, Message> {
    // Calculer le VolumeScale dynamiquement en fonction des bougies visibles
    // Similaire à comment l'axe Y des prix fonctionne
    let volume_scale = {
        let (min_time, max_time) = app.chart_state.viewport.time_scale().time_range();
        let volume_range = app.chart_state.series_manager
            .active_series()
            .next()
            .and_then(|series| {
                // D'abord essayer d'obtenir la plage pour les bougies visibles
                let visible_range = series.data.volume_range_for_time_range(min_time..max_time);
                
                // Si aucune bougie visible, utiliser la plage globale comme fallback
                visible_range.or_else(|| series.data.volume_range())
            })
            .map(|(_min, max)| {
                // Toujours forcer le min à 0 pour les volumes (barres depuis le bas)
                // Cela garantit une visualisation cohérente
                (0.0, max.max(0.0))
            })
            .filter(|(_min, max)| *max > 0.0) // Filtrer les plages invalides
            .unwrap_or((0.0, 1000.0)); // Dernier fallback si aucune série ou plage invalide
        
        let mut scale = VolumeScale::new(volume_range.0, volume_range.1, VOLUME_CHART_HEIGHT);
        scale.set_height(VOLUME_CHART_HEIGHT);
        scale
    };

    // Ligne principale : Tools (gauche) + Chart (centre) + Axe Y (droite)
    let chart_row = row![
        tools_panel(&app.tools_state).map(Message::ToolsPanel),
        chart(&app.chart_state, &app.tools_state, &app.settings_state, &app.chart_style)
            .map(Message::Chart),
        y_axis(&app.chart_state).map(Message::YAxis)
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // Ligne du volume : espace (sous tools) + Volume Chart + Volume Y Axis
    let volume_row = row![
        container(Space::new())
            .width(Length::Fixed(TOOLS_PANEL_WIDTH))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.10))),
                ..Default::default()
            }),
        volume_chart(&app.chart_state, volume_scale.clone()),
        volume_y_axis(volume_scale)
    ]
    .width(Length::Fill)
    .height(Length::Fixed(VOLUME_CHART_HEIGHT));

    // Ligne du bas : espace comblé (sous tools) + Axe X + bouton settings (coin)
    let x_axis_row = row![
        container(Space::new())
            .width(Length::Fixed(TOOLS_PANEL_WIDTH))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.10))),
                ..Default::default()
            }),
        x_axis(&app.chart_state).map(Message::XAxis),
        corner_settings_button()
    ]
    .width(Length::Fill)
    .height(Length::Fixed(X_AXIS_HEIGHT));

    // Layout du composant chart complet : Chart + Volume + X Axis
    column![
        chart_row,
        volume_row,
        x_axis_row
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Handle de redimensionnement pour le panneau de droite
fn right_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    // Largeur plus visible pour le handle
    let handle_width = 6.0;
    horizontal_resize_handle(handle_width, app.panels.right.is_resizing)
}

/// Section à droite du graphique
fn view_right_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.panels.right.visible {
        return container(Space::new())
            .width(Length::Fixed(0.0))
            .height(Length::Fill)
            .into();
    }
    
    let handle_width = 6.0;
    let panel_content_width = app.panels.right.size - handle_width;
    
    let panel_content = container(
        column![
            row![
                text("Panneau de droite")
                    .size(16)
                    .color(Color::WHITE),
                Space::new().width(Length::Fill),
                button("✕")
                    .on_press(Message::ToggleRightPanel)
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let bg_color = match status {
                            button::Status::Hovered => Color::from_rgb(0.5, 0.2, 0.2),
                            _ => Color::from_rgb(0.3, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg_color)),
                            text_color: Color::WHITE,
                            ..Default::default()
                        }
                    })
            ]
            .align_y(iced::Alignment::Center)
            .spacing(10),
            Space::new().height(Length::Fixed(10.0)),
            text("Cette section peut contenir des informations supplémentaires, des indicateurs, ou d'autres contrôles.")
                .size(12)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        ]
        .padding(15)
        .spacing(10)
    )
    .width(Length::Fixed(panel_content_width))
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.10, 0.10, 0.12))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });
    
    row![
        right_panel_resize_handle(app),
        panel_content
    ]
    .width(Length::Fixed(app.panels.right.size))
    .height(Length::Fill)
    .into()
}

/// Handle de redimensionnement pour le panneau du bas
fn bottom_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    // Hauteur plus visible pour le handle
    let handle_height = 6.0;
    vertical_resize_handle(handle_height, app.panels.bottom.is_resizing)
}

/// Section en bas du graphique
fn view_bottom_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.panels.bottom.visible {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(0.0))
            .into();
    }
    
    let panel_content = container(
        column![
            row![
                text("Panneau du bas")
                    .size(16)
                    .color(Color::WHITE),
                Space::new().width(Length::Fill),
                button("✕")
                    .on_press(Message::ToggleBottomPanel)
                    .padding([4, 8])
                    .style(|_theme, status| {
                        let bg_color = match status {
                            button::Status::Hovered => Color::from_rgb(0.5, 0.2, 0.2),
                            _ => Color::from_rgb(0.3, 0.15, 0.15),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg_color)),
                            text_color: Color::WHITE,
                            ..Default::default()
                        }
                    })
            ]
            .align_y(iced::Alignment::Center)
            .spacing(10),
            Space::new().height(Length::Fixed(10.0)),
            text("Cette section peut contenir des statistiques, des logs, ou d'autres informations contextuelles.")
                .size(12)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        ]
        .padding(15)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.10, 0.10, 0.12))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });
    
    column![
        bottom_panel_resize_handle(app),
        panel_content
    ]
    .width(Length::Fill)
    .height(Length::Fixed(app.panels.bottom.size))
    .into()
}

/// Vue principale de l'application
pub fn view_main(app: &ChartApp) -> Element<'_, Message> {
    // Récupérer le symbole de la série active pour le titre
    let title_text = app.chart_state.series_manager
        .active_series()
        .next()
        .map(|series| series.symbol.clone())
        .unwrap_or_else(|| String::from("Chart Candlestick"));
    
    // Header avec titre, bouton de configuration et select box de séries
    let header = container(
        row![
            text(title_text)
                .size(24)
                .color(Color::WHITE),
            Space::new().width(Length::Fill),
            button("⚙️ Provider")
                .on_press(Message::OpenProviderConfig)
                .style(|_theme, status| {
                    let bg_color = match status {
                        button::Status::Hovered => Color::from_rgb(0.2, 0.2, 0.25),
                        _ => Color::from_rgb(0.15, 0.15, 0.18),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                }),
            Space::new().width(Length::Fixed(10.0)),
            series_select_box(&app.chart_state.series_manager).map(Message::SeriesPanel)
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
    )
    .width(Length::Fill)
    .padding(15)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        ..Default::default()
    });

    // Zone principale : Composant chart + Panneau de droite (si visible)
    let main_content = if app.panels.right.visible {
        row![
            view_chart_component(app),
            view_right_panel(app)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        row![
            view_chart_component(app)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    };

    // Layout complet : Header + Zone principale + Panneau du bas
    column![
        header,
        main_content,
        view_bottom_panel(app)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Vue des settings (style du graphique)
pub fn view_settings(app: &ChartApp) -> Element<'_, Message> {
    let fields = color_fields();
    let presets = preset_colors();
    
    let editing_style = app.editing_style.as_ref();
    
    // Titre
    let title = text("Style du graphique")
        .size(20)
        .color(Color::WHITE);

    // Séparateur
    let separator = || container(Space::new().height(1))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.35))),
            ..Default::default()
        });

    // Liste des champs de couleur
    let mut color_rows = column![].spacing(10);
    
    for (index, field) in fields.iter().enumerate() {
        let current_color = if let Some(style) = editing_style {
            (field.get)(style)
        } else {
            SerializableColor::from_iced(Color::WHITE)
        };
        
        let color_box = container(text(""))
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(25.0))
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(current_color.to_iced())),
                border: iced::Border {
                    color: Color::WHITE,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            });

        let color_btn = button(color_box)
            .on_press(Message::ToggleColorPicker(index))
            .padding(0)
            .style(|_theme, _status| button::Style {
                background: None,
                ..Default::default()
            });

        let label = text(field.label)
            .size(14)
            .color(Color::from_rgb(0.8, 0.8, 0.8));

        let field_row = row![
            label,
            Space::new().width(Length::Fill),
            color_btn
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        color_rows = color_rows.push(field_row);

        // Si ce color picker est ouvert, afficher les presets
        if app.editing_color_index == Some(index) {
            let mut presets_row = row![].spacing(5);
            for preset in &presets {
                let preset_color = *preset;
                let preset_box = container(text(""))
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(preset_color.to_iced())),
                        border: iced::Border {
                            color: Color::from_rgb(0.5, 0.5, 0.5),
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    });
                
                let preset_btn = button(preset_box)
                    .on_press(Message::SelectColor(index, preset_color))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..Default::default()
                    });
                
                presets_row = presets_row.push(preset_btn);
            }
            
            let presets_container = container(
                scrollable(presets_row).direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default().width(5).scroller_width(5)
                ))
            )
            .padding(10)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.35),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            });
            
            color_rows = color_rows.push(presets_container);
        }
    }

    // Boutons Apply/Cancel
    let apply_btn = button(
        text("Appliquer").size(14)
    )
    .on_press(Message::ApplySettings)
    .padding([8, 20])
    .style(|_theme, _status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.2))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let cancel_btn = button(
        text("Annuler").size(14)
    )
    .on_press(Message::CancelSettings)
    .padding([8, 20])
    .style(|_theme, _status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.5, 0.2, 0.2))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let buttons_row = row![
        Space::new().width(Length::Fill),
        cancel_btn,
        apply_btn
    ]
    .spacing(10);

    // Toggle pour l'auto-scroll
    let auto_scroll_enabled = editing_style
        .map(|s| s.auto_scroll_enabled)
        .unwrap_or(true);
    
    let auto_scroll_toggle = row![
        checkbox(auto_scroll_enabled)
            .on_toggle(|_| Message::ToggleAutoScroll),
        text("Défilement automatique vers les dernières données")
            .size(14)
            .color(Color::from_rgb(0.8, 0.8, 0.8))
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // Layout complet
    let content = column![
        title,
        Space::new().height(10),
        separator(),
        Space::new().height(10),
        scrollable(color_rows).height(Length::Fill),
        Space::new().height(10),
        separator(),
        Space::new().height(10),
        auto_scroll_toggle,
        Space::new().height(10),
        separator(),
        Space::new().height(10),
        buttons_row
    ]
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
            ..Default::default()
        })
        .into()
}

/// Vue de la configuration des providers
pub fn view_provider_config(app: &ChartApp) -> Element<'_, Message> {
    let title = text("Configuration des Providers")
        .size(20)
        .color(Color::WHITE);

    let mut provider_list = column![].spacing(15);

    for provider_type in ProviderType::all() {
        let is_active = app.provider_config.active_provider == provider_type;
        let provider_name = text(provider_type.display_name())
            .size(16)
            .color(if is_active { Color::from_rgb(0.4, 0.8, 1.0) } else { Color::WHITE });
        
        let description = text(provider_type.description())
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        // Token input
        let current_token = app.editing_provider_token
            .get(&provider_type)
            .cloned()
            .unwrap_or_default();
        
        let token_input = text_input("API Token (optionnel)", &current_token)
            .on_input(move |token| Message::UpdateProviderToken(provider_type, token))
            .padding(8);

        // Bouton de sélection
        let select_btn = if is_active {
            button(text("✓ Actif").size(12))
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.2))),
                    text_color: Color::WHITE,
                    ..Default::default()
                })
        } else {
            button(text("Sélectionner").size(12))
                .on_press(Message::SelectProvider(provider_type))
                .style(|_theme, status| {
                    let bg_color = match status {
                        button::Status::Hovered => Color::from_rgb(0.2, 0.2, 0.25),
                        _ => Color::from_rgb(0.15, 0.15, 0.18),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                })
        };

        let provider_card = container(
            column![
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
            .spacing(8)
            .padding(15)
        )
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
            border: iced::Border {
                color: if is_active {
                    Color::from_rgb(0.4, 0.8, 1.0)
                } else {
                    Color::from_rgb(0.2, 0.2, 0.25)
                },
                width: if is_active { 2.0 } else { 1.0 },
                radius: 8.0.into(),
            },
            ..Default::default()
        });

        provider_list = provider_list.push(provider_card);
    }

    let apply_btn = button(
        text("Appliquer").size(14)
    )
    .on_press(Message::ApplyProviderConfig)
    .padding([8, 20])
    .style(|_theme, _status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.2))),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let cancel_btn = button(
        text("Annuler").size(14)
    )
    .on_press(Message::CancelProviderConfig)
    .padding([8, 20])
    .style(|_theme, status| {
        let bg_color = match status {
            button::Status::Hovered => Color::from_rgb(0.3, 0.2, 0.2),
            _ => Color::from_rgb(0.25, 0.15, 0.15),
        };
        button::Style {
            background: Some(iced::Background::Color(bg_color)),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

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
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.10))),
            ..Default::default()
        })
        .into()
}

