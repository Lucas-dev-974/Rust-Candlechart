//! Canvas pour l'axe Y des volumes
//! 
//! Affiche les valeurs de volume sur l'axe vertical à droite du graphique de volume.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text};
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;

use crate::finance_chart::scale::VolumeScale;
use crate::finance_chart::render::calculate_nice_step;
use crate::finance_chart::axis::Y_AXIS_WIDTH;

/// Style pour l'axe des volumes
struct AxisStyle {
    background_color: Color,
    text_color: Color,
    text_size: f32,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgb(0.08, 0.08, 0.10),
            text_color: Color::from_rgb(0.7, 0.7, 0.7),
            text_size: 11.0,
        }
    }
}

/// Program pour l'axe Y des volumes
pub struct VolumeAxisProgram {
    volume_scale: VolumeScale,
}

impl VolumeAxisProgram {
    pub fn new(volume_scale: VolumeScale) -> Self {
        Self { volume_scale }
    }
}

impl<Message> Program<Message> for VolumeAxisProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let style = AxisStyle::default();

        // Fond de l'axe
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            style.background_color,
        );

        // Calculer les niveaux de volume
        let (min_volume, max_volume) = self.volume_scale.volume_range();
        let volume_range = max_volume - min_volume;
        let volume_step = calculate_nice_step(volume_range);

        // Trouver le premier niveau rond >= min_volume
        let first_volume = (min_volume / volume_step).ceil() * volume_step;

        let mut volume = first_volume;
        while volume <= max_volume {
            let y = self.volume_scale.volume_to_y(volume);

            // Ne dessiner que si visible
            if y >= 0.0 && y <= bounds.height {
                let label = if volume_step >= 1.0 {
                    format!("{:.0}", volume)
                } else if volume_step >= 0.1 {
                    format!("{:.1}", volume)
                } else {
                    format!("{:.2}", volume)
                };

                let text = Text {
                    content: label,
                    position: Point::new(5.0, y - 6.0),
                    color: style.text_color,
                    size: iced::Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }

            volume += volume_step;
        }

        vec![frame.into_geometry()]
    }
}

/// Crée un élément canvas pour l'axe Y des volumes
pub fn volume_y_axis<Message: 'static>(volume_scale: VolumeScale) -> Element<'static, Message> {
    Canvas::new(VolumeAxisProgram::new(volume_scale))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill)
        .into()
}

