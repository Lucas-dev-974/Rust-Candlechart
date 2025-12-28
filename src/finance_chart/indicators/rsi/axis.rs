//! Canvas pour l'axe Y du RSI
//! 
//! Affiche les valeurs du RSI (0-100) sur l'axe vertical à droite du graphique RSI.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text, Path, Stroke};
use iced::widget::stack;
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use iced::Pixels;

use crate::finance_chart::axis::{Y_AXIS_WIDTH, AxisStyle};
use crate::finance_chart::state::ChartState;
use super::data::{calculate_all_rsi_values, get_last_rsi_value};

/// Program pour l'axe Y du RSI
pub struct RSIAxisProgram {
    height: f32,
}

impl RSIAxisProgram {
    pub fn new(height: f32) -> Self {
        Self { height }
    }
}

/// Program pour l'overlay du label RSI qui suit la valeur
pub struct RSILabelOverlayProgram {
    height: f32,
    rsi_value: Option<f64>,
}

impl RSILabelOverlayProgram {
    pub fn new(height: f32, rsi_value: Option<f64>) -> Self {
        Self { height, rsi_value }
    }
}

impl<Message> Program<Message> for RSILabelOverlayProgram {
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

        if let Some(rsi_value) = self.rsi_value {
            let normalized_rsi = (rsi_value / 100.0).clamp(0.0, 1.0);
            let y = self.height * (1.0 - normalized_rsi as f32);
            
            if y >= 0.0 && y <= bounds.height {
                // Dessiner une ligne horizontale pour indiquer la position
                let line_path = Path::new(|builder| {
                    builder.move_to(Point::new(0.0, y));
                    builder.line_to(Point::new(bounds.width, y));
                });
                frame.stroke(
                    &line_path,
                    Stroke::default()
                        .with_color(Color::from_rgba(0.0, 0.8, 1.0, 0.3)) // Cyan transparent
                        .with_width(1.0),
                );
                
                // Dessiner le label à gauche de l'axe
                let text = Text {
                    content: format!("RSI: {:.2}", rsi_value),
                    position: Point::new(5.0, y - 6.0),
                    color: Color::from_rgb(0.0, 0.8, 1.0), // Cyan
                    size: Pixels(11.0),
                    ..Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
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

/// Crée un widget canvas pour l'axe Y du RSI avec overlay de la valeur RSI actuelle
pub fn rsi_y_axis<'a>(chart_state: &'a ChartState, height: f32) -> Element<'a, crate::app::messages::Message> {
    let all_rsi_values = calculate_all_rsi_values(chart_state);
    
    let current_rsi = all_rsi_values.as_ref()
        .and_then(|values| get_last_rsi_value(chart_state, Some(values)));
    
    let axis = Canvas::new(RSIAxisProgram::new(height))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    let overlay = Canvas::new(RSILabelOverlayProgram::new(height, current_rsi))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    stack![
        axis,
        overlay
    ]
    .width(Length::Fixed(Y_AXIS_WIDTH))
    .height(Length::Fill)
    .into()
}

