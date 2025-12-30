//! Canvas pour l'axe Y du RSI
//! 
//! Affiche les valeurs du RSI (0-100) sur l'axe vertical à droite du graphique RSI.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text};
use iced::{Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use iced::Pixels;

use crate::finance_chart::axis::{Y_AXIS_WIDTH, AxisStyle};
use crate::finance_chart::state::ChartState;

/// Program pour l'axe Y du RSI
pub struct RSIAxisProgram {
    height: f32,
}

impl RSIAxisProgram {
    pub fn new(height: f32) -> Self {
        Self { height }
    }
}



impl<Message> Program<Message> for RSIAxisProgram {
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

        // Fond
        let background = iced::widget::canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, style.background_color);

        // Le RSI varie de 0 à 100
        let max_rsi = 100.0;
        let step = 20.0;
        
        let mut rsi_values = Vec::new();
        let mut value = 0.0;
        while value <= max_rsi {
            rsi_values.push(value);
            value += step;
        }

        // Dessiner les labels
        for rsi_value in rsi_values {
            let normalized_rsi = (rsi_value / 100.0_f64).clamp(0.0_f64, 1.0_f64);
            let y = self.height * (1.0 - normalized_rsi as f32);
            
            if y >= 0.0 && y <= bounds.height {
                let text = Text {
                    content: format!("{:.0}", rsi_value),
                    position: Point::new(bounds.width - 5.0 - 15.0, y),
                    color: style.text_color,
                    size: Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Crée un widget canvas pour l'axe Y du RSI (sans overlay)
pub fn rsi_y_axis<'a>(_chart_state: &'a ChartState, height: f32) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(RSIAxisProgram::new(height))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fixed(height))
        .into()
}

