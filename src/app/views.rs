//! Vues de l'application Iced
//!
//! Ce module contient toutes les méthodes de rendu (view) pour les différentes fenêtres
//! de l'application : fenêtre principale, settings, et configuration des providers.

use iced::widget::{button, column, container, row, text, scrollable, Space, checkbox, text_input, mouse_area, stack};
use iced::{Element, Length, Color};
use crate::finance_chart::{
    chart, x_axis, y_axis, tools_panel, series_select_box,
    volume_chart, volume_y_axis,
    rsi_chart, rsi_y_axis,
    macd_chart, macd_y_axis,
    X_AXIS_HEIGHT, TOOLS_PANEL_WIDTH,
    settings::{color_fields, preset_colors, SerializableColor},
    ProviderType, VolumeScale,
};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    resize_handle::{horizontal_resize_handle, vertical_resize_handle, volume_resize_handle},
    bottom_panel_sections::BottomPanelSection,
    view_styles::{self, colors},
    drag_overlay,
    constants::INDICATORS_PANEL_WIDTH,
};

/// Fonction helper pour le bouton de settings dans le coin
fn corner_settings_button() -> Element<'static, Message> {
    button("⚙️")
        .on_press(Message::OpenSettings)
        .padding(8)
        .style(view_styles::icon_button_style)
        .into()
}

/// Définition d'un indicateur avec son nom et son état
struct Indicator {
    name: &'static str,
    is_active: bool,
    on_toggle: fn(bool) -> Message,
}

/// Vue de l'onglet d'indicateurs
fn indicators_panel(app: &ChartApp) -> Element<'_, Message> {
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
fn chart_with_indicators_overlay<'a>(chart_content: Element<'a, Message>, app: &'a ChartApp) -> Element<'a, Message> {
    if app.indicators_panel_open {
        // Utiliser stack pour superposer l'overlay sur le graphique
        // Le premier élément est en dessous, le dernier est au-dessus
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

/// Handle de redimensionnement pour le panneau de volume
fn volume_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    // Hauteur plus visible pour le handle
    let handle_height = 6.0;
    volume_resize_handle(handle_height, app.panels.volume.is_resizing)
}

/// Bouton de fermeture du volume chart en overlay
fn volume_chart_close_button() -> Element<'static, Message> {
    container(
        button("🗑️")
            .on_press(Message::ToggleVolumePanel)
            .padding(4)
            .style(view_styles::icon_button_style)
    )
    .padding(5)
    .into()
}

/// Handle de redimensionnement pour le panneau RSI
fn rsi_panel_resize_handle_helper(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    use crate::app::resize_handle::rsi_panel_resize_handle;
    rsi_panel_resize_handle(handle_height, app.panels.rsi.is_resizing)
}

/// Bouton de fermeture du graphique RSI (icône poubelle)
fn rsi_chart_close_button() -> Element<'static, Message> {
    container(
        button("🗑️")
            .on_press(Message::ToggleRSIPanel)
            .padding(4)
            .style(view_styles::icon_button_style)
    )
    .padding(5)
    .into()
}

/// Handle de redimensionnement pour le panneau MACD
fn macd_panel_resize_handle_helper(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    use crate::app::resize_handle::macd_panel_resize_handle;
    macd_panel_resize_handle(handle_height, app.panels.macd.is_resizing)
}

/// Bouton de fermeture du graphique MACD (icône poubelle)
fn macd_chart_close_button() -> Element<'static, Message> {
    container(
        button("🗑️")
            .on_press(Message::ToggleMACDPanel)
            .padding(4)
            .style(view_styles::icon_button_style)
    )
    .padding(5)
    .into()
}

/// Composant qui regroupe toutes les sections du graphique
/// (tools_panel, chart, y_axis, x_axis, volume_chart, volume_y_axis, rsi_chart, rsi_y_axis)
fn view_chart_component(app: &ChartApp) -> Element<'_, Message> {
    // Ligne principale : Tools (gauche) + Chart (centre) + Axe Y (droite)
    // L'onglet d'indicateurs sera en overlay par-dessus
    let panel_focused = app.panels.has_focused_panel();
    let chart_row = row![
        tools_panel(&app.tools_state, app.indicators_panel_open).map(Message::ToolsPanel),
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

    // Vérifier si le volume panel est visible et snappé
    let is_volume_visible = app.panels.volume.visible;
    let is_volume_snapped = app.panels.volume.is_snapped();
    
    // Utiliser la hauteur dynamique du panneau de volume
    let volume_height = app.panels.volume.size;
    
    // Handle de redimensionnement du volume (entre le chart et le volume)
    let volume_resize_handle = mouse_area(
        volume_panel_resize_handle(app)
    )
    .on_enter(Message::SetVolumePanelFocus(true))
    .on_exit(Message::SetVolumePanelFocus(false));

    // Construire le layout avec le volume si visible
    let volume_row = if is_volume_visible && !is_volume_snapped {
        // Utiliser toute la hauteur pour le volume chart (plus de header séparé)
        let volume_chart_height = volume_height;
        
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
            
            let mut scale = VolumeScale::new(volume_range.0, volume_range.1, volume_chart_height);
            scale.set_height(volume_chart_height);
            scale
        };
        
        // Volume chart avec overlay du bouton de fermeture
        let volume_chart_with_overlay = stack![
            // Le volume chart (en dessous)
            row![
                container(Space::new())
                    .width(Length::Fixed(TOOLS_PANEL_WIDTH))
                    .height(Length::Fill)
                    .style(view_styles::dark_background_style),
                volume_chart(&app.chart_state, volume_scale.clone()),
                volume_y_axis(volume_scale)
            ]
            .width(Length::Fill)
            .height(Length::Fixed(volume_chart_height)),
            // Bouton de fermeture en overlay (au-dessus, positionné en haut à gauche)
            container(
                row![
                    Space::new().width(Length::Fixed(TOOLS_PANEL_WIDTH)),
                    volume_chart_close_button()
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
        ]
        .width(Length::Fill)
        .height(Length::Fixed(volume_chart_height));
        
        // Ligne du volume : Volume Chart avec overlay + Volume Y Axis
        Some(volume_chart_with_overlay
            .width(Length::Fill)
            .height(Length::Fixed(volume_height)))
    } else {
        None
    };

    // Vérifier si le RSI panel est visible
    let is_rsi_visible = app.panels.rsi.visible;
    let is_rsi_snapped = app.panels.rsi.is_snapped();
    let rsi_height = app.panels.rsi.size;
    
    // Handle de redimensionnement du RSI (entre le volume et le RSI, ou entre le chart et le RSI)
    let rsi_resize_handle = mouse_area(
        rsi_panel_resize_handle_helper(app)
    )
    .on_enter(Message::SetRSIPanelFocus(true))
    .on_exit(Message::SetRSIPanelFocus(false));
    
    // Construire le layout avec le volume et le RSI si visibles
    let mut layout_items: Vec<Element<'_, Message>> = vec![chart_row.into()];
    
    // Ajouter le volume si visible
    if is_volume_visible {
        layout_items.push(volume_resize_handle.into());
        if let Some(vol_row) = volume_row {
            layout_items.push(vol_row.into());
        }
    }
    
    // Ajouter le RSI si visible
    if is_rsi_visible {
        layout_items.push(rsi_resize_handle.into());
        
        if !is_rsi_snapped {
            // RSI chart avec overlay du bouton de fermeture
            let rsi_chart_with_overlay = stack![
                // Le RSI chart (en dessous)
                row![
                    container(Space::new())
                        .width(Length::Fixed(TOOLS_PANEL_WIDTH))
                        .height(Length::Fill)
                        .style(view_styles::dark_background_style),
                    rsi_chart(&app.chart_state),
                    rsi_y_axis(&app.chart_state, rsi_height)
                ]
                .width(Length::Fill)
                .height(Length::Fixed(rsi_height)),
                // Bouton de fermeture en overlay (au-dessus, positionné en haut à gauche)
                container(
                    row![
                        Space::new().width(Length::Fixed(TOOLS_PANEL_WIDTH)),
                        rsi_chart_close_button()
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Top)
            ]
            .width(Length::Fill)
            .height(Length::Fixed(rsi_height));
            
            layout_items.push(rsi_chart_with_overlay.into());
        }
    }
    
    // Vérifier si le MACD panel est visible
    let is_macd_visible = app.panels.macd.visible;
    let is_macd_snapped = app.panels.macd.is_snapped();
    let macd_height = app.panels.macd.size;
    
    // Handle de redimensionnement du MACD
    let macd_resize_handle = mouse_area(
        macd_panel_resize_handle_helper(app)
    )
    .on_enter(Message::SetMACDPanelFocus(true))
    .on_exit(Message::SetMACDPanelFocus(false));
    
    // Ajouter le MACD si visible
    if is_macd_visible {
        layout_items.push(macd_resize_handle.into());
        
        if !is_macd_snapped {
            // MACD chart avec overlay du bouton de fermeture
            let macd_chart_with_overlay = stack![
                // Le MACD chart (en dessous)
                row![
                    container(Space::new())
                        .width(Length::Fixed(TOOLS_PANEL_WIDTH))
                        .height(Length::Fill)
                        .style(view_styles::dark_background_style),
                    macd_chart(&app.chart_state),
                    macd_y_axis(&app.chart_state)
                ]
                .width(Length::Fill)
                .height(Length::Fixed(macd_height)),
                // Bouton de fermeture en overlay (au-dessus, positionné en haut à gauche)
                container(
                    row![
                        Space::new().width(Length::Fixed(TOOLS_PANEL_WIDTH)),
                        macd_chart_close_button()
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Top)
            ]
            .width(Length::Fill)
            .height(Length::Fixed(macd_height));
            
            layout_items.push(macd_chart_with_overlay.into());
        }
    }
    
    // Ajouter l'axe X en dernier
    layout_items.push(x_axis_row.into());
    
    // Créer le layout principal
    let mut main_chart_layout = column![];
    for item in layout_items {
        main_chart_layout = main_chart_layout.push(item);
    }
    let main_chart_layout = main_chart_layout
        .width(Length::Fill)
        .height(Length::Fill);
    
    // Ajouter l'overlay d'indicateurs si ouvert
    chart_with_indicators_overlay(main_chart_layout.into(), app)
}

/// Handle de redimensionnement pour le panneau de droite
fn right_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    // Largeur plus visible pour le handle
    let handle_width = 6.0;
    horizontal_resize_handle(handle_width, app.panels.right.is_resizing)
}

/// Header du panneau de droite avec les boutons de sections
fn right_panel_header(app: &ChartApp, panel_width: f32) -> Element<'_, Message> {
    let sections = &app.bottom_panel_sections.right_panel_sections;
    let header_height = 40.0;
    
    if sections.is_empty() {
        // Zone de drop vide
        let drop_text = if app.dragging_section.is_some() && app.drag_over_right_panel {
            "Relâchez pour déposer"
        } else if app.dragging_section.is_some() {
            "Déposez ici"
        } else {
            "Glissez une section ici"
        };
        
        let text_color = if app.dragging_section.is_some() && app.drag_over_right_panel {
            Color::from_rgb(0.0, 0.8, 0.0)
        } else {
            colors::TEXT_SECONDARY
        };
        
        return container(
            text(drop_text)
                .size(12)
                .color(text_color)
        )
        .width(Length::Fixed(panel_width))
        .height(Length::Fixed(header_height))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme| {
            let border_color = if app.dragging_section.is_some() && app.drag_over_right_panel {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.2, 0.2, 0.25)
            };
            container::Style {
                background: Some(iced::Background::Color(colors::BACKGROUND_DARK)),
                border: iced::Border {
                    color: border_color,
                    width: if app.dragging_section.is_some() && app.drag_over_right_panel { 2.0 } else { 1.0 },
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into();
    }
    
    let mut buttons_row = row![].spacing(5);
    
    for &section in sections {
        let is_active = app.bottom_panel_sections.active_right_section == Some(section);
        let is_dragging = app.dragging_section == Some(section);
        let section_name = section.display_name();
        
        // Style du bouton
        let section_content = container(
            text(section_name).size(11).color(if is_active || is_dragging {
                colors::TEXT_PRIMARY
            } else {
                colors::TEXT_SECONDARY
            })
        )
        .padding([4, 8])
        .style(move |_theme| {
            let bg_color = if is_dragging {
                Color::from_rgb(0.25, 0.35, 0.5)
            } else if is_active {
                Color::from_rgb(0.2, 0.25, 0.35)
            } else {
                Color::from_rgb(0.12, 0.12, 0.15)
            };
            container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    color: if is_active || is_dragging {
                        Color::from_rgb(0.3, 0.4, 0.6)
                    } else {
                        Color::from_rgb(0.2, 0.2, 0.25)
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });
        
        // Envelopper dans mouse_area pour le drag & drop (vers le panneau du bas)
        let section_button = mouse_area(section_content)
            .on_press(Message::StartDragSection(section));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    container(
        scrollable(
            buttons_row.padding([0, 5])
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default().width(3).scroller_width(3)
        ))
    )
    .width(Length::Fixed(panel_width))
    .height(Length::Fixed(header_height))
    .style(view_styles::header_container_style)
    .into()
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
    
    // Header avec les boutons de sections
    let header = right_panel_header(app, panel_content_width);
    
    // Contenu du panneau
    let panel_content = if let Some(section) = app.bottom_panel_sections.active_right_section {
        // Afficher la section active
        container(
            scrollable(
                view_bottom_panel_section_content(app, section)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .padding(10)
        .style(view_styles::panel_container_style)
    } else if app.bottom_panel_sections.has_right_panel_sections() {
        // Des sections existent mais aucune n'est active (ne devrait pas arriver)
        container(
            text("Sélectionnez une section")
                .size(12)
                .color(colors::TEXT_SECONDARY)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(view_styles::panel_container_style)
    } else {
        // Pas de sections - zone de drop
        let drop_zone_text = if app.dragging_section.is_some() && app.drag_over_right_panel {
            "Relâchez pour déposer ici"
        } else if app.dragging_section.is_some() {
            "Déposez ici"
        } else {
            "Glissez une section du panneau du bas ici"
        };
        
        let drop_zone_color = if app.dragging_section.is_some() && app.drag_over_right_panel {
            Color::from_rgb(0.0, 0.8, 0.0)
        } else {
            colors::TEXT_SECONDARY
        };
        
        container(
            column![
                text("Panneau de droite")
                    .size(16)
                    .color(colors::TEXT_PRIMARY),
                Space::new().height(Length::Fixed(10.0)),
                text(drop_zone_text)
                    .size(12)
                    .color(drop_zone_color)
            ]
            .spacing(5)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme| {
            let border_color = if app.dragging_section.is_some() && app.drag_over_right_panel {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.2, 0.2, 0.25)
            };
            let mut style = view_styles::panel_container_style(_theme);
            style.border = iced::Border {
                color: border_color,
                width: if app.dragging_section.is_some() && app.drag_over_right_panel { 2.0 } else { 1.0 },
                radius: 6.0.into(),
            };
            style
        })
    };
    
    // Zone de drop avec gestion des événements de drag & drop
    let drop_zone = mouse_area(
        column![
            header,
            panel_content
        ]
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
    )
    .on_enter(Message::DragEnterRightPanel)
    .on_exit(Message::DragExitRightPanel)
    .on_press(if app.dragging_section.is_some() {
        Message::EndDragSection
    } else {
        Message::ClearPanelFocus
    });
    
    // Englober tout le panneau (poignée + contenu) dans mouse_area
    mouse_area(
        row![
            right_panel_resize_handle(app),
            drop_zone
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

/// Header du panneau du bas avec les boutons de sections (avec drag & drop)
fn bottom_panel_header(app: &ChartApp) -> Element<'_, Message> {
    let header_height = 40.0;
    let mut buttons_row = row![].spacing(5);
    
    // N'afficher que les sections qui sont dans le panneau du bas
    let bottom_sections = app.bottom_panel_sections.bottom_panel_sections();
    let is_bottom_empty = bottom_sections.is_empty();
    
    for section in bottom_sections {
        let is_active = app.bottom_panel_sections.active_bottom_section == section;
        let is_dragging = app.dragging_section == Some(section);
        let section_name = section.display_name();
        
        // Bouton principal de la section avec drag & drop
        // On utilise un container stylé au lieu d'un button pour que mouse_area capture le clic
        let section_content = container(
            row![
                text(section_name).size(12).color(if is_active || is_dragging {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                }),
                // Indicateur de drag
                text(" ⋮⋮").size(10).color(Color::from_rgb(0.4, 0.4, 0.5))
            ]
            .align_y(iced::Alignment::Center)
        )
        .padding([6, 12])
        .style(move |_theme| {
            let bg_color = if is_dragging {
                Color::from_rgb(0.25, 0.35, 0.5) // Bleu pendant le drag
            } else if is_active {
                Color::from_rgb(0.2, 0.25, 0.35) // Actif
            } else {
                Color::from_rgb(0.12, 0.12, 0.15) // Normal
            };
            container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    color: if is_active || is_dragging {
                        Color::from_rgb(0.3, 0.4, 0.6)
                    } else {
                        Color::from_rgb(0.2, 0.2, 0.25)
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });
        
        // Envelopper dans mouse_area pour gérer le clic (démarre le drag ET sélectionne)
        let section_button = mouse_area(section_content)
            .on_press(Message::StartDragSection(section));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    // Zone de drop si on drague depuis le panneau de droite
    if is_bottom_empty {
        // Afficher une zone de drop si le panneau du bas est vide
        let drop_text = if app.dragging_section.is_some() && !app.drag_over_right_panel {
            "Déposez ici"
        } else {
            "Aucune section"
        };
        
        buttons_row = buttons_row.push(
            text(drop_text)
                .size(12)
                .color(colors::TEXT_SECONDARY)
        );
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

/// Crée une ligne d'information (label + valeur)
fn create_info_row(label: &str, value: String, value_color: Option<Color>) -> Element<'_, Message> {
    row![
        text(label)
            .size(12)
            .color(colors::TEXT_SECONDARY),
        Space::new().width(Length::Fill),
        text(value)
            .size(12)
            .color(value_color.unwrap_or(colors::TEXT_PRIMARY))
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Crée la section Solde et Équité
fn create_balance_section(app: &ChartApp) -> Element<'_, Message> {
    let info = &app.account_info;
    
    let section_card_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };
    
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
    
    let section_card_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };
    
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
    
    let section_card_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };
    
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
    
    let section_card_style = |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.2, 0.2, 0.25),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    };
    
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
fn view_bottom_panel_account(app: &ChartApp) -> Element<'_, Message> {
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
        true // En mode paper, considéré comme "connecté" (pas de connexion nécessaire)
    };
    
    // Couleur de la pastille selon le statut
    let status_color = if is_connected {
        Color::from_rgb(0.0, 0.8, 0.0) // Vert si connecté
    } else {
        Color::from_rgb(0.8, 0.0, 0.0) // Rouge si non connecté
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
                
                // Section Solde et Équité
                create_balance_section(app),
                
                Space::new().height(Length::Fixed(20.0)),
                
                // Section Marge
                create_margin_section(app),
                
                Space::new().height(Length::Fixed(20.0)),
                
                // Section P&L
                create_pnl_section(app),
                
                Space::new().height(Length::Fixed(20.0)),
                
                // Section Positions et Risque
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

/// Affiche le contenu d'une section spécifique (utilisé pour le panneau de droite)
fn view_bottom_panel_section_content(app: &ChartApp, section: BottomPanelSection) -> Element<'_, Message> {
    match section {
        BottomPanelSection::Overview => view_bottom_panel_overview(app),
        BottomPanelSection::Logs => view_bottom_panel_logs(app),
        BottomPanelSection::Indicators => view_bottom_panel_indicators(app),
        BottomPanelSection::Orders => view_bottom_panel_orders(app),
        BottomPanelSection::Account => view_bottom_panel_account(app),
    }
}

/// Affiche le contenu de la section active
fn view_bottom_panel_content(app: &ChartApp) -> Element<'_, Message> {
    let active_section = app.bottom_panel_sections.active_bottom_section;
    // Ne pas afficher la section si elle est dans le panneau de droite
    if app.bottom_panel_sections.is_section_in_right_panel(active_section) {
        // Afficher un message indiquant que la section est dans le panneau de droite
        container(
            column![
                text(format!("La section \"{}\" est actuellement dans le panneau de droite.", active_section.display_name()))
                    .size(14)
                    .color(colors::TEXT_SECONDARY),
                Space::new().height(Length::Fixed(10.0)),
                text("Cliquez sur ← dans le header pour la ramener ici.")
                    .size(12)
                    .color(colors::TEXT_SECONDARY),
            ]
            .padding(20)
            .spacing(10)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        view_bottom_panel_section_content(app, active_section)
    }
}

/// Section en bas du graphique avec gestion du drag & drop global
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
    
    // Détermine si on drag depuis le panneau de droite (pour afficher le feedback)
    let dragging_from_right = app.dragging_section.is_some() && app.drag_from_right_panel;
    
    // Englober tout le panneau (poignée + header + contenu) dans mouse_area
    let panel = mouse_area(
        column![
            bottom_panel_resize_handle(app),
            bottom_panel_header(app),
            container(view_bottom_panel_content(app))
                .width(Length::Fill)
                .height(Length::Fixed(panel_content_height))
        ]
        .width(Length::Fill)
        .height(Length::Fixed(app.panels.bottom.size))
    )
    .on_enter(Message::SetBottomPanelFocus(true))
    .on_exit(Message::SetBottomPanelFocus(false));
    
    // Ajouter on_press pour terminer le drag si on drague depuis la droite
    if dragging_from_right {
        panel.on_press(Message::EndDragSection).into()
    } else {
        panel.into()
    }
}

/// Vue principale de l'application avec gestion globale du drag & drop
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
    let main_layout = column![
        header,
        main_content,
        view_bottom_panel(app)
    ]
    .width(Length::Fill)
    .height(Length::Fill);
    
    // Si on drag une section, ajouter l'overlay qui suit la souris
    if let Some(section) = app.dragging_section {
        // Utiliser la position du drag ou une position par défaut
        let position = app.drag_position.unwrap_or(iced::Point::new(0.0, 0.0));
        
        // Créer un container avec le layout principal et le canvas overlay par-dessus
        // Le canvas doit être au-dessus de tout pour capturer les événements de souris
        // On utilise une column avec le layout principal et le canvas overlay en dernier
        container(
            column![
                main_layout,
                // Canvas overlay qui capture les événements de souris et affiche le composant visuel
                // Il sera rendu au-dessus de tout car il est en dernier dans la column
                drag_overlay::drag_overlay(section, position)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        main_layout.into()
    }
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

