//! Widget pour gérer le drag & drop visuel des sections
//!
//! Utilise un canvas pour détecter les événements de drag & drop.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Color, Element, Event, Length, Point, Rectangle, mouse};
use iced::mouse::Cursor;
use crate::app::bottom_panel_sections::BottomPanelSection;

/// État du drag d'une section
#[derive(Debug, Clone, Default)]
pub struct SectionDragState {
    pub is_dragging: bool,
    pub drag_start: Option<Point>,
}

/// Programme canvas pour le bouton de section avec drag & drop
pub struct SectionDragButton {
    section: BottomPanelSection,
    is_active: bool,
    is_in_right_panel: bool,
    is_dragging: bool,
    width: f32,
    height: f32,
}

impl SectionDragButton {
    pub fn new(section: BottomPanelSection, is_active: bool, is_in_right_panel: bool, is_dragging: bool, width: f32, height: f32) -> Self {
        Self {
            section,
            is_active,
            is_in_right_panel,
            is_dragging,
            width,
            height,
        }
    }
}

impl<Message> Program<Message> for SectionDragButton
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = SectionDragState;

    fn draw(&self, state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Couleur de fond selon l'état
        let bg_color = if state.is_dragging || self.is_dragging {
            Color::from_rgb(0.3, 0.5, 0.7) // Bleu pendant le drag
        } else if self.is_active {
            Color::from_rgb(0.2, 0.3, 0.4) // Actif
        } else {
            Color::from_rgb(0.15, 0.15, 0.18) // Normal
        };
        
        let button_path = Path::rectangle(Point::new(0.0, 0.0), bounds.size());
        frame.fill(&button_path, bg_color);
        
        // Bordure
        let border_color = if self.is_active {
            Color::from_rgb(0.4, 0.6, 0.8)
        } else {
            Color::from_rgb(0.25, 0.25, 0.3)
        };
        frame.stroke(&button_path, Stroke::default().with_color(border_color).with_width(1.0));
        
        // Note: Le texte doit être rendu par le widget parent, pas par le canvas
        // Le canvas sert uniquement à gérer les événements de drag
        
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    state.is_dragging = true;
                    if let Some(pos) = cursor.position_in(bounds) {
                        state.drag_start = Some(pos);
                        // Initialiser la position du drag avec la position de la souris
                        return Some(iced::widget::canvas::Action::batch(vec![
                            iced::widget::canvas::Action::publish(
                                Message::from(crate::app::messages::Message::UpdateDragPosition(pos))
                            ),
                            iced::widget::canvas::Action::publish(
                                Message::from(crate::app::messages::Message::StartDragSection(self.section))
                            ),
                        ]));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.drag_start = None;
                    // Ne pas terminer le drag ici, laisser l'overlay global le faire
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    // Mettre à jour la position du drag
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::UpdateDragPosition(pos))
                        ));
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
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            if state.is_dragging || self.is_dragging {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::Pointer
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un bouton de section avec drag & drop
pub fn section_drag_button(section: BottomPanelSection, is_active: bool, is_in_right_panel: bool, is_dragging: bool, width: f32, height: f32) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(SectionDragButton::new(section, is_active, is_in_right_panel, is_dragging, width, height))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

