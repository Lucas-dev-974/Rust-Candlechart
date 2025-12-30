//! Panneau du bas

use iced::widget::{container, mouse_area, row, stack, text, Space, column};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    ui::vertical_resize_handle,
    view_styles::{self, colors},
};
use super::super::context_menu_capture::context_menu_capture;
use super::sections::view_section_content;

/// Handle de redimensionnement pour le panneau du bas
fn bottom_panel_resize_handle(app: &ChartApp) -> Element<'_, Message> {
    let handle_height = 6.0;
    vertical_resize_handle(handle_height, app.ui.panels.bottom.is_resizing)
}

/// Header du panneau du bas avec les boutons de sections
fn bottom_panel_header(app: &ChartApp) -> Element<'_, Message> {
    let header_height = 40.0;
    let mut buttons_row = row![].spacing(5);
    
    let bottom_sections = app.ui.bottom_panel_sections.bottom_panel_sections();
    let is_bottom_empty = bottom_sections.is_empty();
    
    for section in bottom_sections {
        let is_active = app.ui.bottom_panel_sections.active_bottom_section == section;
        let section_name = section.display_name();
        
        let section_content = container(
            text(section_name).size(12).color(if is_active {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
            })
        )
        .padding([6, 12])
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
        .on_press(Message::SelectBottomSection(section));
        
        buttons_row = buttons_row.push(section_button);
    }
    
    if is_bottom_empty {
        buttons_row = buttons_row.push(
            text("Aucune section").size(12).color(colors::TEXT_SECONDARY)
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

/// Affiche le contenu de la section active
fn view_bottom_panel_content(app: &ChartApp) -> Element<'_, Message> {
    let active_section = app.ui.bottom_panel_sections.active_bottom_section;
    
    if app.ui.bottom_panel_sections.is_section_in_right_panel(active_section) {
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
        view_section_content(app, active_section)
    }
}

/// Section en bas du graphique
pub fn view_bottom_panel(app: &ChartApp) -> Element<'_, Message> {
    if !app.ui.panels.bottom.visible {
        return container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(0.0))
            .into();
    }
    
    let handle_height = 6.0;
    let is_snapped = app.ui.panels.bottom.is_snapped();
    
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
    let panel_content_height = app.ui.panels.bottom.size - handle_height - header_height;
    
    mouse_area(
        column![
            bottom_panel_resize_handle(app),
            bottom_panel_header(app),
            container(view_bottom_panel_content(app))
                .width(Length::Fill)
                .height(Length::Fixed(panel_content_height))
        ]
        .width(Length::Fill)
        .height(Length::Fixed(app.ui.panels.bottom.size))
    )
    .on_enter(Message::SetBottomPanelFocus(true))
    .on_exit(Message::SetBottomPanelFocus(false))
    .into()
}

