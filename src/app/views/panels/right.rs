//! Panneau de droite

use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    ui::horizontal_resize_handle,
    view_styles::{self, colors},
};
use super::super::context_menu_capture::context_menu_capture;
use super::sections::view_section_content;

/// Handle de redimensionnement pour le panneau de droite
fn right_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    let handle_width = 6.0;
    horizontal_resize_handle(handle_width, app.ui.panels.right.is_resizing)
}

/// Header du panneau de droite avec les boutons de sections
fn right_panel_header(app: &ChartApp, panel_width: f32) -> Element<'_, Message> {
    let sections = &app.ui.bottom_panel_sections.right_panel_sections;
    let header_height = 40.0;
    
    if sections.is_empty() {
        return container(text("Aucune section").size(12).color(colors::TEXT_SECONDARY))
            .width(Length::Fixed(panel_width))
            .height(Length::Fixed(header_height))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(view_styles::header_container_style)
            .into();
    }
    
    let mut buttons_row = row![].spacing(5);
    
    for &section in sections {
        let is_active = app.ui.bottom_panel_sections.active_right_section == Some(section);
        let section_name = section.display_name();
        
        let section_content = container(
            text(section_name).size(11).color(if is_active {
                colors::TEXT_PRIMARY
            } else {
                colors::TEXT_SECONDARY
            })
        )
        .padding([4, 8])
        .style(move |_theme| {
            let bg_color = if is_active {
                Color::from_rgb(0.2, 0.25, 0.35)
            } else {
                Color::from_rgb(0.12, 0.12, 0.15)
            };
            container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    color: if is_active {
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
        
        // Utiliser un widget canvas pour capturer la position du curseur lors du clic droit
        let section_clone = section;
        let section_button = mouse_area(
            stack![
                section_content,
                context_menu_capture(section_clone)
            ]
        )
        .on_press(Message::SelectRightSection(section));
        
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

/// Menu contextuel pour les sections
pub fn section_context_menu(app: &ChartApp) -> Element<'_, Message> {
    if let Some((section, _position)) = &app.ui.section_context_menu {
        let is_in_right_panel = app.ui.bottom_panel_sections.is_section_in_right_panel(*section);
        
        let menu_items = column![
            if is_in_right_panel {
                button("Déplacer vers le bas")
                    .on_press(Message::MoveSectionToBottomPanel(*section))
                    .style(view_styles::icon_button_style)
                    .width(Length::Fill)
            } else {
                button("Déplacer vers la droite")
                    .on_press(Message::MoveSectionToRightPanel(*section))
                    .style(view_styles::icon_button_style)
                    .width(Length::Fill)
            },
        ]
        .spacing(4)
        .padding(8);
        
        container(menu_items)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.18))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.3, 0.35),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fixed(180.0))
            .into()
    } else {
        container(Space::new()).into()
    }
}

/// Section à droite du graphique
pub fn view_right_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.ui.panels.right.visible {
        return container(Space::new())
            .width(Length::Fixed(0.0))
            .height(Length::Fill)
            .into();
    }
    
    let handle_width = 6.0;
    let is_snapped = app.ui.panels.right.is_snapped();
    
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
    
    let panel_content_width = app.ui.panels.right.size - handle_width;
    let header = right_panel_header(app, panel_content_width);
    
    let panel_content = if let Some(section) = app.ui.bottom_panel_sections.active_right_section {
        container(
            scrollable(view_section_content(app, section))
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .padding(10)
        .style(view_styles::panel_container_style)
    } else if app.ui.bottom_panel_sections.has_right_panel_sections() {
        container(
            text("Sélectionnez une section").size(12).color(colors::TEXT_SECONDARY)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(view_styles::panel_container_style)
    } else {
        container(
            column![
                text("Panneau de droite").size(16).color(colors::TEXT_PRIMARY),
                Space::new().height(Length::Fixed(10.0)),
                text("Aucune section").size(12).color(colors::TEXT_SECONDARY)
            ]
            .spacing(5)
        )
        .width(Length::Fixed(panel_content_width))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(view_styles::panel_container_style)
    };
    
    let drop_zone = mouse_area(
        column![header, panel_content]
            .width(Length::Fixed(panel_content_width))
            .height(Length::Fill)
    )
    .on_press(Message::ClearPanelFocus);
    
    mouse_area(
        row![right_panel_resize_handle(app), drop_zone]
            .width(Length::Fixed(app.ui.panels.right.size))
            .height(Length::Fill)
    )
    .on_enter(Message::SetRightPanelFocus(true))
    .on_exit(Message::SetRightPanelFocus(false))
    .into()
}

