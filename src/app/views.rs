//! Vues de l'application Iced
//!
//! Ce module contient toutes les méthodes de rendu (view) pour les différentes fenêtres
//! de l'application : fenêtre principale, settings, et configuration des providers.

use iced::widget::{button, column, container, row, text, scrollable, Space, checkbox, text_input, mouse_area};
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
    bottom_panel_sections::{BottomPanelSection, BottomPanelSectionsState},
    view_styles::{self, colors},
};

/// Fonction helper pour le bouton de settings dans le coin
fn corner_settings_button() -> Element<'static, Message> {
    button("⚙️")
        .on_press(Message::OpenSettings)
        .padding(8)
        .style(view_styles::icon_button_style)
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
    let panel_focused = app.panels.has_focused_panel();
    let chart_row = row![
        tools_panel(&app.tools_state).map(Message::ToolsPanel),
        mouse_area(
            container(
                chart(&app.chart_state, &app.tools_state, &app.settings_state, &app.chart_style, panel_focused)
                    .map(Message::Chart)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_enter(Message::ClearPanelFocus),
        y_axis(&app.chart_state).map(Message::YAxis)
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // Ligne du volume : espace (sous tools) + Volume Chart + Volume Y Axis
    let volume_row = row![
        container(Space::new())
            .width(Length::Fixed(TOOLS_PANEL_WIDTH))
            .height(Length::Fill)
            .style(view_styles::dark_background_style),
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
            .style(view_styles::dark_background_style),
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
    let is_snapped = app.panels.right.is_snapped();
    
    // Si le panneau est snappé, on affiche seulement la poignée (avec protection mouse_area)
    if is_snapped {
        return mouse_area(
            row![
                right_panel_resize_handle(app)
            ]
            .width(Length::Fixed(handle_width))
            .height(Length::Fill)
        )
        .on_enter(Message::SetRightPanelFocus(true))
        .on_exit(Message::SetRightPanelFocus(false))
        .into();
    }
    
    let panel_content_width = app.panels.right.size - handle_width;
    
    let panel_content = container(
        column![
            row![
                text("Panneau de droite")
                    .size(16)
                    .color(colors::TEXT_PRIMARY),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(10),
            Space::new().height(Length::Fixed(10.0)),
            text("Cette section peut contenir des informations supplémentaires, des indicateurs, ou d'autres contrôles.")
                .size(12)
                .color(colors::TEXT_SECONDARY)
        ]
        .padding(15)
        .spacing(10)
    )
    .width(Length::Fixed(panel_content_width))
    .height(Length::Fill)
    .style(view_styles::panel_container_style);
    
    // Englober tout le panneau (poignée + contenu) dans mouse_area
    mouse_area(
        row![
            right_panel_resize_handle(app),
            panel_content
        ]
        .width(Length::Fixed(app.panels.right.size))
        .height(Length::Fill)
    )
    .on_enter(Message::SetRightPanelFocus(true))
    .on_exit(Message::SetRightPanelFocus(false))
    .into()
}

/// Handle de redimensionnement pour le panneau du bas
fn bottom_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    // Hauteur plus visible pour le handle
    let handle_height = 6.0;
    vertical_resize_handle(handle_height, app.panels.bottom.is_resizing)
}

/// Header du panneau du bas avec les boutons de sections
fn bottom_panel_header(sections_state: &BottomPanelSectionsState) -> Element<'_, Message> {
    let header_height = 40.0;
    let mut buttons_row = row![].spacing(5);
    
    for section in BottomPanelSection::all() {
        let is_active = sections_state.active_section == section;
        let section_name = section.display_name();
        
        let section_button = button(text(section_name).size(12))
            .on_press(Message::SelectBottomPanelSection(section))
            .padding([6, 12])
            .style(view_styles::tab_button_style(is_active));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    container(
        row![
            buttons_row,
            Space::new().width(Length::Fill),
        ]
        .align_y(iced::Alignment::Center)
        .padding([0, 10])
    )
    .width(Length::Fill)
    .height(Length::Fixed(header_height))
    .style(view_styles::header_container_style)
    .into()
}

/// Helper pour créer une vue de section de panneau simple
fn simple_panel_section<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
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

/// Vue pour la section "Vue d'ensemble"
fn view_bottom_panel_overview(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Vue d'ensemble",
        "Cette section affiche une vue d'ensemble des statistiques et informations principales."
    )
}

/// Vue pour la section "Logs"
fn view_bottom_panel_logs(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Logs",
        "Cette section affiche les logs de l'application."
    )
}

/// Vue pour la section "Indicateurs"
fn view_bottom_panel_indicators(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Indicateurs techniques",
        "Cette section affiche les indicateurs techniques et analyses."
    )
}

/// Vue pour la section "Ordres"
fn view_bottom_panel_orders(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Ordres et Trades",
        "Cette section affiche les ordres et trades."
    )
}

/// Vue pour la section "Compte"
fn view_bottom_panel_account(app: &ChartApp) -> Element<'_, Message> {
    let is_demo_mode = app.account_type.is_demo();
    let is_real_mode = app.account_type.is_real();
    let account_type_name = app.account_type.account_type.display_name();
    let account_type_desc = app.account_type.account_type.description();
    
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
        true // En mode paper, considéré comme "connecté" (pas de connexion nécessaire)
    };
    
    // Couleur de la pastille selon le statut
    let status_color = if is_connected {
        Color::from_rgb(0.0, 0.8, 0.0) // Vert si connecté
    } else {
        Color::from_rgb(0.8, 0.0, 0.0) // Rouge si non connecté
    };
    
    // Style pour les cartes de section
    let section_card_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };
    
    // Section "Type de compte"
    let account_type_section = container(
        column![
            // En-tête de section
            row![
                text("Type de compte")
                    .size(14)
                    .color(colors::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                // Badge du type de compte
                container(
                    text(account_type_name)
                        .size(11)
                        .color(if is_real_mode {
                            Color::from_rgb(1.0, 0.7, 0.7)
                        } else {
                            Color::from_rgb(0.7, 0.9, 0.7)
                        })
                )
                .padding([3, 8])
                .style(move |_theme| {
                    let is_real = is_real_mode;
                    container::Style {
                        background: Some(iced::Background::Color(if is_real {
                            Color::from_rgb(0.3, 0.15, 0.15)
                        } else {
                            Color::from_rgb(0.15, 0.3, 0.15)
                        })),
                        border: iced::Border {
                            color: if is_real {
                                Color::from_rgb(0.8, 0.4, 0.4)
                            } else {
                                Color::from_rgb(0.4, 0.8, 0.4)
                            },
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(12.0)),
            
            // Description du mode actuel
            container(
                text(account_type_desc)
                    .size(11)
                    .color(if is_real_mode {
                        Color::from_rgb(1.0, 0.7, 0.7)
                    } else {
                        Color::from_rgb(0.7, 0.9, 0.7)
                    })
            )
            .padding([8, 10])
            .style(move |_theme| {
                let is_real = is_real_mode;
                container::Style {
                    background: Some(iced::Background::Color(if is_real {
                        Color::from_rgb(0.25, 0.12, 0.12)
                    } else {
                        Color::from_rgb(0.12, 0.25, 0.12)
                    })),
                    border: iced::Border {
                        color: if is_real {
                            Color::from_rgb(0.6, 0.3, 0.3)
                        } else {
                            Color::from_rgb(0.3, 0.6, 0.3)
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            }),
        ]
        .padding(12)
        .spacing(8)
    )
    .style(section_card_style);
    
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
                                let color = status_color;
                                container::Style {
                                    background: Some(iced::Background::Color(color)),
                                    border: iced::Border {
                                        color: color,
                                        width: 0.0,
                                        radius: 5.0.into(), // Cercle parfait
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
                
                // Section Type de compte
                account_type_section,
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

/// Affiche le contenu de la section active
fn view_bottom_panel_content(app: &ChartApp) -> Element<'_, Message> {
    match app.bottom_panel_sections.active_section {
        BottomPanelSection::Overview => view_bottom_panel_overview(app),
        BottomPanelSection::Logs => view_bottom_panel_logs(app),
        BottomPanelSection::Indicators => view_bottom_panel_indicators(app),
        BottomPanelSection::Orders => view_bottom_panel_orders(app),
        BottomPanelSection::Account => view_bottom_panel_account(app),
    }
}

/// Section en bas du graphique
fn view_bottom_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.panels.bottom.visible {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(0.0))
            .into();
    }
    
    let handle_height = 6.0;
    let is_snapped = app.panels.bottom.is_snapped();
    
    // Si le panneau est snappé, on affiche seulement la poignée (avec protection mouse_area)
    if is_snapped {
        return mouse_area(
            column![
                bottom_panel_resize_handle(app)
            ]
            .width(Length::Fill)
            .height(Length::Fixed(handle_height))
        )
        .on_enter(Message::SetBottomPanelFocus(true))
        .on_exit(Message::SetBottomPanelFocus(false))
        .into();
    }
    
    let header_height = 40.0;
    let panel_content_height = app.panels.bottom.size - handle_height - header_height;
    
    // Englober tout le panneau (poignée + header + contenu) dans mouse_area
    mouse_area(
        column![
            bottom_panel_resize_handle(app),
            bottom_panel_header(&app.bottom_panel_sections),
            container(view_bottom_panel_content(app))
                .width(Length::Fill)
                .height(Length::Fixed(panel_content_height))
        ]
        .width(Length::Fill)
        .height(Length::Fixed(app.panels.bottom.size))
    )
    .on_enter(Message::SetBottomPanelFocus(true))
    .on_exit(Message::SetBottomPanelFocus(false))
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
                .color(colors::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button("⚙️ Provider")
                .on_press(Message::OpenProviderConfig)
                .style(view_styles::icon_button_style),
            Space::new().width(Length::Fixed(10.0)),
            series_select_box(&app.chart_state.series_manager).map(Message::SeriesPanel)
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
    )
    .width(Length::Fill)
    .padding(15)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(colors::BACKGROUND_HEADER)),
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
    .style(view_styles::success_button_style);

    let cancel_btn = button(
        text("Annuler").size(14)
    )
    .on_press(Message::CancelSettings)
    .padding([8, 20])
    .style(view_styles::danger_button_style);

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
            .color(colors::TEXT_TERTIARY)
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
            background: Some(iced::Background::Color(colors::BACKGROUND_HEADER)),
            ..Default::default()
        })
        .into()
}

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
        
        // Récupérer le token actuel (depuis editing ou config)
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
                        Color::from_rgb(0.7, 0.9, 0.7) // Vert si connecté
                    } else {
                        Color::from_rgb(1.0, 0.7, 0.7) // Rouge si non connecté
                    })
            )
            .padding([3, 8])
            .style(move |_theme: &iced::Theme| {
                let connected: bool = is_connected;
                container::Style {
                    background: Some(iced::Background::Color(if connected {
                        Color::from_rgb(0.15, 0.3, 0.15) // Fond vert si connecté
                    } else {
                        Color::from_rgb(0.3, 0.15, 0.15) // Fond rouge si non connecté
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

