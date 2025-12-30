//! Widget canvas pour capturer la position du curseur lors du clic droit

use iced::widget::canvas::{Canvas, Program};
use iced::{Event, Length, Rectangle, mouse};
use crate::app::{messages::Message, state::BottomPanelSection};

/// État du widget de capture du menu contextuel
#[derive(Debug, Default)]
struct ContextMenuCaptureState {}

/// Programme canvas pour capturer la position du curseur lors du clic droit
struct ContextMenuCapture {
    section: BottomPanelSection,
}

impl ContextMenuCapture {
    fn new(section: BottomPanelSection) -> Self {
        Self { section }
    }
}

impl Program<Message> for ContextMenuCapture {
    type State = ContextMenuCaptureState;

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &iced::Theme,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        // Ne rien dessiner, c'est juste un widget invisible pour capturer les événements
        vec![]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if cursor.is_over(bounds) {
                    // Utiliser la position globale du curseur
                    if let Some(global_pos) = cursor.position() {
                        return Some(iced::widget::canvas::Action::publish(
                            Message::OpenSectionContextMenu(self.section, global_pos)
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
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un widget canvas pour capturer la position du curseur lors du clic droit
pub fn context_menu_capture(section: BottomPanelSection) -> iced::Element<'static, Message> {
    Canvas::new(ContextMenuCapture::new(section))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

