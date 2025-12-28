//! Panneaux latéraux (droite et bas)

use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text, Space};
use iced::{Element, Length, Color};
use crate::finance_chart::{
    volume_chart, volume_y_axis,
    rsi_chart, rsi_y_axis,
    macd_chart, macd_y_axis,
    TOOLS_PANEL_WIDTH,
    VolumeScale,
};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    resize_handle::{horizontal_resize_handle, vertical_resize_handle, volume_resize_handle},
    bottom_panel_sections::BottomPanelSection,
    view_styles::{self, colors},
};
use super::helpers::simple_panel_section;
use super::account::view_bottom_panel_account;

// ============================================================================
// Panneaux d'indicateurs (Volume, RSI, MACD)
// ============================================================================

/// Handle de redimensionnement pour le panneau de volume
pub fn volume_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    volume_resize_handle(handle_height, app.panels.volume.is_resizing)
}

/// Bouton de fermeture du volume chart en overlay
pub fn volume_chart_close_button() -> Element<'static, Message> {
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
pub fn rsi_panel_resize_handle_helper(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    use crate::app::resize_handle::rsi_panel_resize_handle;
    rsi_panel_resize_handle(handle_height, app.panels.rsi.is_resizing)
}

/// Bouton de fermeture du graphique RSI
pub fn rsi_chart_close_button() -> Element<'static, Message> {
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
pub fn macd_panel_resize_handle_helper(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    use crate::app::resize_handle::macd_panel_resize_handle;
    macd_panel_resize_handle(handle_height, app.panels.macd.is_resizing)
}

/// Bouton de fermeture du graphique MACD
pub fn macd_chart_close_button() -> Element<'static, Message> {
    container(
        button("🗑️")
            .on_press(Message::ToggleMACDPanel)
            .padding(4)
            .style(view_styles::icon_button_style)
    )
    .padding(5)
    .into()
}

/// Construit le layout avec Volume, RSI, MACD si visibles
pub fn build_indicator_panels<'a>(app: &'a ChartApp, layout_items: &mut Vec<Element<'a, Message>>) {
    // Volume
    let is_volume_visible = app.panels.volume.visible;
    let is_volume_snapped = app.panels.volume.is_snapped();
    let volume_height = app.panels.volume.size;
    
    let volume_resize_handle = mouse_area(volume_panel_resize_handle(app))
        .on_enter(Message::SetVolumePanelFocus(true))
        .on_exit(Message::SetVolumePanelFocus(false));

    if is_volume_visible {
        layout_items.push(volume_resize_handle.into());
        
        if !is_volume_snapped {
            let volume_chart_height = volume_height;
            
            let volume_scale = {
                let (min_time, max_time) = app.chart_state.viewport.time_scale().time_range();
                let volume_range = app.chart_state.series_manager
                    .active_series()
                    .next()
                    .and_then(|series| {
                        let visible_range = series.data.volume_range_for_time_range(min_time..max_time);
                        visible_range.or_else(|| series.data.volume_range())
                    })
                    .map(|(_min, max)| (0.0, max.max(0.0)))
                    .filter(|(_min, max)| *max > 0.0)
                    .unwrap_or((0.0, 1000.0));
                
                let mut scale = VolumeScale::new(volume_range.0, volume_range.1, volume_chart_height);
                scale.set_height(volume_chart_height);
                scale
            };
            
            let volume_chart_with_overlay = stack![
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
            
            layout_items.push(volume_chart_with_overlay.into());
        }
    }

    // RSI
    let is_rsi_visible = app.panels.rsi.visible;
    let is_rsi_snapped = app.panels.rsi.is_snapped();
    let rsi_height = app.panels.rsi.size;
    
    let rsi_resize_handle = mouse_area(rsi_panel_resize_handle_helper(app))
        .on_enter(Message::SetRSIPanelFocus(true))
        .on_exit(Message::SetRSIPanelFocus(false));
    
    if is_rsi_visible {
        layout_items.push(rsi_resize_handle.into());
        
        if !is_rsi_snapped {
            let rsi_chart_with_overlay = stack![
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
    
    // MACD
    let is_macd_visible = app.panels.macd.visible;
    let is_macd_snapped = app.panels.macd.is_snapped();
    let macd_height = app.panels.macd.size;
    
    let macd_resize_handle = mouse_area(macd_panel_resize_handle_helper(app))
        .on_enter(Message::SetMACDPanelFocus(true))
        .on_exit(Message::SetMACDPanelFocus(false));
    
    if is_macd_visible {
        layout_items.push(macd_resize_handle.into());
        
        if !is_macd_snapped {
            let macd_chart_with_overlay = stack![
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
}

// ============================================================================
// Panneau de droite
// ============================================================================

/// Handle de redimensionnement pour le panneau de droite
fn right_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    let handle_width = 6.0;
    horizontal_resize_handle(handle_width, app.panels.right.is_resizing)
}

/// Header du panneau de droite avec les boutons de sections
fn right_panel_header(app: &ChartApp, panel_width: f32) -> Element<'_, Message> {
    let sections = &app.bottom_panel_sections.right_panel_sections;
    let header_height = 40.0;
    
    if sections.is_empty() {
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
        
        return container(text(drop_text).size(12).color(text_color))
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
        
        let section_button = mouse_area(section_content)
            .on_press(Message::StartDragSection(section));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    container(
        scrollable(buttons_row.padding([0, 5]))
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
pub fn view_right_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.panels.right.visible {
        return container(Space::new())
            .width(Length::Fixed(0.0))
            .height(Length::Fill)
            .into();
    }
    
    let handle_width = 6.0;
    let is_snapped = app.panels.right.is_snapped();
    
    if is_snapped {
        return mouse_area(
            row![right_panel_resize_handle(app)]
                .width(Length::Fixed(handle_width))
                .height(Length::Fill)
        )
        .on_enter(Message::SetRightPanelFocus(true))
        .on_exit(Message::SetRightPanelFocus(false))
        .into();
    }
    
    let panel_content_width = app.panels.right.size - handle_width;
    let header = right_panel_header(app, panel_content_width);
    
    let panel_content = if let Some(section) = app.bottom_panel_sections.active_right_section {
        container(
            scrollable(view_bottom_panel_section_content(app, section))
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .padding(10)
        .style(view_styles::panel_container_style)
    } else if app.bottom_panel_sections.has_right_panel_sections() {
        container(
            text("Sélectionnez une section").size(12).color(colors::TEXT_SECONDARY)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(view_styles::panel_container_style)
    } else {
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
                text("Panneau de droite").size(16).color(colors::TEXT_PRIMARY),
                Space::new().height(Length::Fixed(10.0)),
                text(drop_zone_text).size(12).color(drop_zone_color)
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
    
    let drop_zone = mouse_area(
        column![header, panel_content]
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
    
    mouse_area(
        row![right_panel_resize_handle(app), drop_zone]
            .width(Length::Fixed(app.panels.right.size))
            .height(Length::Fill)
    )
    .on_enter(Message::SetRightPanelFocus(true))
    .on_exit(Message::SetRightPanelFocus(false))
    .into()
}

// ============================================================================
// Panneau du bas
// ============================================================================

/// Handle de redimensionnement pour le panneau du bas
fn bottom_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    vertical_resize_handle(handle_height, app.panels.bottom.is_resizing)
}

/// Header du panneau du bas avec les boutons de sections
fn bottom_panel_header(app: &ChartApp) -> Element<'_, Message> {
    let header_height = 40.0;
    let mut buttons_row = row![].spacing(5);
    
    let bottom_sections = app.bottom_panel_sections.bottom_panel_sections();
    let is_bottom_empty = bottom_sections.is_empty();
    
    for section in bottom_sections {
        let is_active = app.bottom_panel_sections.active_bottom_section == section;
        let is_dragging = app.dragging_section == Some(section);
        let section_name = section.display_name();
        
        let section_content = container(
            row![
                text(section_name).size(12).color(if is_active || is_dragging {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                }),
                text(" ⋮⋮").size(10).color(Color::from_rgb(0.4, 0.4, 0.5))
            ]
            .align_y(iced::Alignment::Center)
        )
        .padding([6, 12])
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
        
        let section_button = mouse_area(section_content)
            .on_press(Message::StartDragSection(section));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    if is_bottom_empty {
        let drop_text = if app.dragging_section.is_some() && !app.drag_over_right_panel {
            "Déposez ici"
        } else {
            "Aucune section"
        };
        
        buttons_row = buttons_row.push(
            text(drop_text).size(12).color(colors::TEXT_SECONDARY)
        );
    }
    
    container(
        row![buttons_row, Space::new().width(Length::Fill)]
            .align_y(iced::Alignment::Center)
            .padding([0, 10])
    )
    .width(Length::Fill)
    .height(Length::Fixed(header_height))
    .style(view_styles::header_container_style)
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

/// Affiche le contenu d'une section spécifique
pub fn view_bottom_panel_section_content(app: &ChartApp, section: BottomPanelSection) -> Element<'_, Message> {
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
    
    if app.bottom_panel_sections.is_section_in_right_panel(active_section) {
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

/// Section en bas du graphique
pub fn view_bottom_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.panels.bottom.visible {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(0.0))
            .into();
    }
    
    let handle_height = 6.0;
    let is_snapped = app.panels.bottom.is_snapped();
    
    if is_snapped {
        return mouse_area(
            column![bottom_panel_resize_handle(app)]
                .width(Length::Fill)
                .height(Length::Fixed(handle_height))
        )
        .on_enter(Message::SetBottomPanelFocus(true))
        .on_exit(Message::SetBottomPanelFocus(false))
        .into();
    }
    
    let header_height = 40.0;
    let panel_content_height = app.panels.bottom.size - handle_height - header_height;
    let dragging_from_right = app.dragging_section.is_some() && app.drag_from_right_panel;
    
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
    
    if dragging_from_right {
        panel.on_press(Message::EndDragSection).into()
    } else {
        panel.into()
    }
}

