//! Widget pour gérer le redimensionnement des panneaux
//!
//! Utilise un canvas simple pour détecter les événements de souris et gérer le drag.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size, mouse};
use iced::mouse::Cursor;

/// État du handle de redimensionnement
#[derive(Debug, Clone, Default)]
pub struct ResizeHandleState {
    pub is_dragging: bool,
    pub drag_start: Option<f32>,
}

/// Programme canvas pour le handle de redimensionnement horizontal (panneau de droite)
pub struct HorizontalResizeHandle {
    width: f32,
    is_resizing: bool,
}

impl HorizontalResizeHandle {
    pub fn new(width: f32, is_resizing: bool) -> Self {
        Self { width, is_resizing }
    }
}

impl<Message> Program<Message> for HorizontalResizeHandle
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ResizeHandleState;

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner le handle avec une couleur plus visible
        let handle_color = if self.is_resizing {
            Color::from_rgb(0.4, 0.6, 0.9)
        } else {
            // Couleur plus claire pour être visible
            Color::from_rgb(0.3, 0.3, 0.35)
        };
        
        let handle = Path::rectangle(Point::new(0.0, 0.0), Size::new(self.width, bounds.height));
        frame.fill(&handle, handle_color);
        
        // Ajouter une bordure pour mieux voir le handle
        let border_color = if self.is_resizing {
            Color::from_rgb(0.5, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 0.4, 0.45)
        };
        
        // Bordure droite
        let border = Path::line(
            Point::new(self.width - 0.5, 0.0),
            Point::new(self.width - 0.5, bounds.height)
        );
        frame.stroke(&border, Stroke::default().with_color(border_color).with_width(1.0));
        
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
                    if let Some(global_pos) = cursor.position() {
                        state.drag_start = Some(global_pos.x);
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::StartResizeRightPanel(global_pos.x))
                        ));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.drag_start = None;
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndResizeRightPanel)
                ));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    return Some(iced::widget::canvas::Action::publish(
                        Message::from(crate::app::messages::Message::UpdateResizeRightPanel(position.x))
                    ));
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
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Programme canvas pour le handle de redimensionnement vertical (panneau du bas)
pub struct VerticalResizeHandle {
    height: f32,
    is_resizing: bool,
}

impl VerticalResizeHandle {
    pub fn new(height: f32, is_resizing: bool) -> Self {
        Self { height, is_resizing }
    }
}

impl<Message> Program<Message> for VerticalResizeHandle
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ResizeHandleState;

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner le handle avec une couleur plus visible
        let handle_color = if self.is_resizing {
            Color::from_rgb(0.4, 0.6, 0.9)
        } else {
            // Couleur plus claire pour être visible
            Color::from_rgb(0.3, 0.3, 0.35)
        };
        
        let handle = Path::rectangle(Point::new(0.0, 0.0), Size::new(bounds.width, self.height));
        frame.fill(&handle, handle_color);
        
        // Ajouter une bordure pour mieux voir le handle
        let border_color = if self.is_resizing {
            Color::from_rgb(0.5, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 0.4, 0.45)
        };
        
        // Bordure basse
        let border = Path::line(
            Point::new(0.0, self.height - 0.5),
            Point::new(bounds.width, self.height - 0.5)
        );
        frame.stroke(&border, Stroke::default().with_color(border_color).with_width(1.0));
        
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
                    if let Some(global_pos) = cursor.position() {
                        state.drag_start = Some(global_pos.y);
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::StartResizeBottomPanel(global_pos.y))
                        ));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.drag_start = None;
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndResizeBottomPanel)
                ));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    return Some(iced::widget::canvas::Action::publish(
                        Message::from(crate::app::messages::Message::UpdateResizeBottomPanel(position.y))
                    ));
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
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un handle de redimensionnement horizontal
pub fn horizontal_resize_handle(width: f32, is_resizing: bool) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(HorizontalResizeHandle::new(width, is_resizing))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .into()
}

/// Programme canvas pour le handle de redimensionnement vertical du volume chart
pub struct VolumeResizeHandle {
    height: f32,
    is_resizing: bool,
}

impl VolumeResizeHandle {
    pub fn new(height: f32, is_resizing: bool) -> Self {
        Self { height, is_resizing }
    }
}

impl<Message> Program<Message> for VolumeResizeHandle
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ResizeHandleState;

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner le handle avec une couleur plus visible
        let handle_color = if self.is_resizing {
            Color::from_rgb(0.4, 0.6, 0.9)
        } else {
            // Couleur plus claire pour être visible
            Color::from_rgb(0.3, 0.3, 0.35)
        };
        
        let handle = Path::rectangle(Point::new(0.0, 0.0), Size::new(bounds.width, self.height));
        frame.fill(&handle, handle_color);
        
        // Ajouter une bordure pour mieux voir le handle
        let border_color = if self.is_resizing {
            Color::from_rgb(0.5, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 0.4, 0.45)
        };
        
        // Bordure basse
        let border = Path::line(
            Point::new(0.0, self.height - 0.5),
            Point::new(bounds.width, self.height - 0.5)
        );
        frame.stroke(&border, Stroke::default().with_color(border_color).with_width(1.0));
        
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
                    if let Some(global_pos) = cursor.position() {
                        state.drag_start = Some(global_pos.y);
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::StartResizeVolumePanel(global_pos.y))
                        ));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.drag_start = None;
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndResizeVolumePanel)
                ));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    return Some(iced::widget::canvas::Action::publish(
                        Message::from(crate::app::messages::Message::UpdateResizeVolumePanel(position.y))
                    ));
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
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un handle de redimensionnement vertical
pub fn vertical_resize_handle(height: f32, is_resizing: bool) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(VerticalResizeHandle::new(height, is_resizing))
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

/// Crée un handle de redimensionnement vertical pour le volume chart
pub fn volume_resize_handle(height: f32, is_resizing: bool) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(VolumeResizeHandle::new(height, is_resizing))
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

/// Programme canvas pour le handle de redimensionnement vertical du RSI chart
pub struct RSIPanelResizeHandle {
    height: f32,
    is_resizing: bool,
}

impl RSIPanelResizeHandle {
    pub fn new(height: f32, is_resizing: bool) -> Self {
        Self { height, is_resizing }
    }
}

impl<Message> Program<Message> for RSIPanelResizeHandle
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ResizeHandleState;

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner le handle avec une couleur plus visible
        let handle_color = if self.is_resizing {
            Color::from_rgb(0.4, 0.6, 0.9)
        } else {
            // Couleur plus claire pour être visible
            Color::from_rgb(0.3, 0.3, 0.35)
        };
        
        let handle = Path::rectangle(Point::new(0.0, 0.0), Size::new(bounds.width, self.height));
        frame.fill(&handle, handle_color);
        
        // Ajouter une bordure pour mieux voir le handle
        let border_color = if self.is_resizing {
            Color::from_rgb(0.5, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 0.4, 0.45)
        };
        
        // Bordure basse
        let border = Path::line(
            Point::new(0.0, self.height - 0.5),
            Point::new(bounds.width, self.height - 0.5)
        );
        frame.stroke(&border, Stroke::default().with_color(border_color).with_width(1.0));
        
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
                    if let Some(global_pos) = cursor.position() {
                        state.drag_start = Some(global_pos.y);
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::StartResizeRSIPanel(global_pos.y))
                        ));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.drag_start = None;
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndResizeRSIPanel)
                ));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    return Some(iced::widget::canvas::Action::publish(
                        Message::from(crate::app::messages::Message::UpdateResizeRSIPanel(position.y))
                    ));
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
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un handle de redimensionnement vertical pour le RSI chart
pub fn rsi_panel_resize_handle(height: f32, is_resizing: bool) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(RSIPanelResizeHandle::new(height, is_resizing))
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

/// Handle de redimensionnement vertical pour le panneau MACD
pub struct MACDPanelResizeHandle {
    height: f32,
    is_resizing: bool,
}

impl MACDPanelResizeHandle {
    pub fn new(height: f32, is_resizing: bool) -> Self {
        Self { height, is_resizing }
    }
}

impl<Message> Program<Message> for MACDPanelResizeHandle
where
    Message: Clone + From<crate::app::messages::Message>,
{
    type State = ResizeHandleState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        let handle_color = if self.is_resizing {
            Color::from_rgb(0.4, 0.6, 0.9)
        } else {
            Color::from_rgb(0.3, 0.3, 0.35)
        };
        
        let handle = Path::rectangle(Point::new(0.0, 0.0), Size::new(bounds.width, self.height));
        frame.fill(&handle, handle_color);
        
        let border_color = if self.is_resizing {
            Color::from_rgb(0.5, 0.7, 1.0)
        } else {
            Color::from_rgb(0.4, 0.4, 0.45)
        };
        
        // Bordure basse
        let border = Path::line(
            Point::new(0.0, self.height - 0.5),
            Point::new(bounds.width, self.height - 0.5)
        );
        frame.stroke(&border, Stroke::default().with_color(border_color).with_width(1.0));
        
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
                    if let Some(global_pos) = cursor.position() {
                        state.drag_start = Some(global_pos.y);
                        return Some(iced::widget::canvas::Action::publish(
                            Message::from(crate::app::messages::Message::StartResizeMACDPanel(global_pos.y))
                        ));
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_dragging = false;
                state.drag_start = None;
                return Some(iced::widget::canvas::Action::publish(
                    Message::from(crate::app::messages::Message::EndResizeMACDPanel)
                ));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if state.is_dragging {
                    return Some(iced::widget::canvas::Action::publish(
                        Message::from(crate::app::messages::Message::UpdateResizeMACDPanel(position.y))
                    ));
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
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un handle de redimensionnement vertical pour le MACD chart
pub fn macd_panel_resize_handle(height: f32, is_resizing: bool) -> Element<'static, crate::app::messages::Message> {
    Canvas::new(MACDPanelResizeHandle::new(height, is_resizing))
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

