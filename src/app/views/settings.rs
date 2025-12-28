//! Vue des settings (style du graphique)

use iced::widget::{button, checkbox, column, container, row, scrollable, text, Space};
use iced::{Element, Length, Color};
use crate::finance_chart::settings::{color_fields, preset_colors, SerializableColor};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::{self, colors},
};
use super::helpers::separator;

/// Vue des settings (style du graphique)
pub fn view_settings(app: &ChartApp) -> Element<'_, Message> {
    let fields = color_fields();
    let presets = preset_colors();
    
    let editing_style = app.editing_style.as_ref();
    
    // Titre
    let title = text("Style du graphique")
        .size(20)
        .color(Color::WHITE);

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

