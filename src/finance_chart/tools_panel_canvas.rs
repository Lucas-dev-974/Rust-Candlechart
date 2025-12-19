//! Canvas séparé pour le panel d'outils (à gauche du graphique)
//!
//! Architecture Elm : émet des messages pour les mutations d'état,
//! reçoit des références immuables pour le rendu.

use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Program, Action};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size};
use iced::mouse;

use super::tools_canvas::{Tool, ToolsState};
use super::messages::ToolsPanelMessage;

/// Largeur du panel d'outils
pub const TOOLS_PANEL_WIDTH: f32 = 45.0;

/// État local du widget pour le hover (UI uniquement)
#[derive(Debug, Clone, Default)]
pub struct ToolsPanelState {
    pub hovered_button: Option<usize>,
}

/// Program pour le panel d'outils
/// Reçoit une référence immutable, émet des messages
pub struct ToolsPanelProgram<'a> {
    tools_state: &'a ToolsState,
}

impl<'a> ToolsPanelProgram<'a> {
    pub fn new(tools_state: &'a ToolsState) -> Self {
        Self { tools_state }
    }
}

impl<'a> Program<ToolsPanelMessage> for ToolsPanelProgram<'a> {
    type State = ToolsPanelState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Couleurs
        let bg_color = Color::from_rgb(0.08, 0.08, 0.10);
        let button_color = Color::from_rgba(0.15, 0.15, 0.18, 1.0);
        let button_hover_color = Color::from_rgba(0.25, 0.25, 0.28, 1.0);
        let button_selected_color = Color::from_rgba(0.2, 0.5, 0.8, 1.0);
        let icon_color = Color::from_rgba(0.7, 0.7, 0.7, 1.0);
        let border_color = Color::from_rgba(0.2, 0.2, 0.22, 1.0);

        // Fond du panel
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg_color);

        // Bordure droite
        let border = Path::new(|builder| {
            builder.move_to(Point::new(bounds.width - 1.0, 0.0));
            builder.line_to(Point::new(bounds.width - 1.0, bounds.height));
        });
        let stroke = canvas::Stroke::default()
            .with_color(border_color)
            .with_width(1.0);
        frame.stroke(&border, stroke);

        // Outils
        let tools = [Tool::Rectangle, Tool::HorizontalLine];
        
        let button_size = 32.0;
        let padding = (bounds.width - button_size) / 2.0;
        let start_y = 15.0;
        let spacing = 8.0;

        for (index, tool) in tools.iter().enumerate() {
            let y = start_y + (index as f32) * (button_size + spacing);
            let is_hovered = state.hovered_button == Some(index);
            let is_selected = self.tools_state.selected_tool == Some(*tool);

            // Couleur du fond
            let bg = if is_selected {
                button_selected_color
            } else if is_hovered {
                button_hover_color
            } else {
                button_color
            };

            // Fond du bouton
            let button_rect = Path::rectangle(
                Point::new(padding, y),
                Size::new(button_size, button_size),
            );
            frame.fill(&button_rect, bg);

            // Icône
            let cx = padding + button_size / 2.0;
            let cy = y + button_size / 2.0;

            match tool {
                Tool::Rectangle => {
                    let size = 14.0;
                    let icon_rect = Path::rectangle(
                        Point::new(cx - size / 2.0, cy - size / 2.0),
                        Size::new(size, size),
                    );
                    let icon_stroke = canvas::Stroke::default()
                        .with_color(icon_color)
                        .with_width(1.5);
                    frame.stroke(&icon_rect, icon_stroke);
                }
                Tool::HorizontalLine => {
                    let half_width = 11.0;
                    let arrow_size = 3.0;

                    let line = Path::new(|builder| {
                        builder.move_to(Point::new(cx - half_width, cy));
                        builder.line_to(Point::new(cx + half_width, cy));
                        
                        builder.move_to(Point::new(cx - half_width + arrow_size, cy - arrow_size));
                        builder.line_to(Point::new(cx - half_width, cy));
                        builder.line_to(Point::new(cx - half_width + arrow_size, cy + arrow_size));
                        
                        builder.move_to(Point::new(cx + half_width - arrow_size, cy - arrow_size));
                        builder.line_to(Point::new(cx + half_width, cy));
                        builder.line_to(Point::new(cx + half_width - arrow_size, cy + arrow_size));
                    });
                    
                    let icon_stroke = canvas::Stroke::default()
                        .with_color(icon_color)
                        .with_width(1.5);
                    frame.stroke(&line, icon_stroke);
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        panel_state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<ToolsPanelMessage>> {
        let button_size = 32.0;
        let padding = (bounds.width - button_size) / 2.0;
        let start_y = 15.0;
        let spacing = 8.0;
        let tools_count = 2;

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let old_hovered = panel_state.hovered_button;
                panel_state.hovered_button = None;

                if let Some(pos) = cursor.position_in(bounds) {
                    for i in 0..tools_count {
                        let y = start_y + (i as f32) * (button_size + spacing);
                        if pos.x >= padding && pos.x <= padding + button_size &&
                           pos.y >= y && pos.y <= y + button_size {
                            panel_state.hovered_button = Some(i);
                            break;
                        }
                    }
                }

                if old_hovered != panel_state.hovered_button {
                    return Some(Action::request_redraw());
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let tools = [Tool::Rectangle, Tool::HorizontalLine];
                    
                    for (i, tool) in tools.iter().enumerate() {
                        let y = start_y + (i as f32) * (button_size + spacing);
                        if pos.x >= padding && pos.x <= padding + button_size &&
                           pos.y >= y && pos.y <= y + button_size {
                            // Émettre le message au lieu de muter directement
                            return Some(Action::publish(ToolsPanelMessage::ToggleTool { tool: *tool }));
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.hovered_button.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un élément canvas pour le panel d'outils
/// Retourne un Element qui émet ToolsPanelMessage
pub fn tools_panel<'a>(tools_state: &'a ToolsState) -> Element<'a, ToolsPanelMessage> {
    Canvas::new(ToolsPanelProgram::new(tools_state))
        .width(Length::Fixed(TOOLS_PANEL_WIDTH))
        .height(Length::Fill)
        .into()
}
