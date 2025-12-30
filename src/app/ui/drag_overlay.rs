//! Canvas overlay pour afficher le composant visuel qui suit la souris pendant le drag

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke, Text};
use iced::{Color, Element, Event, Length, Point, Rectangle, mouse, Pixels};
use iced::mouse::Cursor;
use crate::app::state::BottomPanelSection;

/// Programme canvas pour l'overlay de drag
pub struct DragOverlay {
    section: BottomPanelSection,
    position: Point,
}

impl DragOverlay {
    pub fn new(section: BottomPanelSection, position: Point) -> Self {
        Self {
            section,
            position,
        }
    }
}

impl<Message> Program<Message> for DragOverlay
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Calculer la position relative dans les bounds
        let x = self.position.x.min(bounds.width).max(0.0);
        let y = self.position.y.min(bounds.height).max(0.0);
        
        // Taille du composant visuel
        let width = 120.0;
        let height = 30.0;
        
        // Ajuster la position pour que le composant soit centré sur la souris
        let rect_x = x - width / 2.0;
        let rect_y = y - height / 2.0;
        
        // Dessiner un rectangle avec ombre pour l'effet de drag
        let shadow_offset = 4.0;
        
        // Ombre
        let shadow_path = Path::rectangle(
            Point::new(rect_x + shadow_offset, rect_y + shadow_offset),
            iced::Size::new(width, height)
        );
        frame.fill(&shadow_path, Color::from_rgba(0.0, 0.0, 0.0, 0.3));
        
        // Rectangle principal
        let rect_path = Path::rectangle(
            Point::new(rect_x, rect_y),
            iced::Size::new(width, height)
        );
        frame.fill(&rect_path, Color::from_rgb(0.25, 0.35, 0.5));
        
        // Bordure
        frame.stroke(
            &rect_path,
            Stroke::default()
                .with_color(Color::from_rgb(0.4, 0.6, 0.8))
                .with_width(2.0)
        );
        
        // Texte de la section (centré)
        let section_name = self.section.display_name();
        let text = Text {
            content: section_name.to_string(),
            position: Point::new(x, y),
            color: Color::WHITE,
            size: Pixels(12.0),
            ..Text::default()
        };
        frame.fill_text(text);
        
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                // Utiliser la position absolue de la souris dans la fenêtre
                // Convertir en position relative dans les bounds du canvas
                let pos = Point::new(
                    position.x - bounds.x,
                    position.y - bounds.y
                );
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::UpdateDragPosition(pos))
                ));
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Terminer le drag quand on relâche le bouton
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndDragSection)
                ));
            }
            _ => {}
        }
        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::Grabbing
    }
}

/// Crée un canvas overlay pour le drag
pub fn drag_overlay(section: BottomPanelSection, position: Point) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(DragOverlay::new(section, position))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

