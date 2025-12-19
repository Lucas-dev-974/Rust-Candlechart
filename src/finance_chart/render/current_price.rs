//! Rendu de la ligne de prix courant

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point};

use super::super::viewport::Viewport;

/// Style pour la ligne de prix courant
pub struct CurrentPriceStyle {
    pub line_color: Color,
    pub line_width: f32,
    pub dash_length: f32,
    pub gap_length: f32,
}

impl Default for CurrentPriceStyle {
    fn default() -> Self {
        Self {
            line_color: Color::from_rgba(0.2, 0.6, 1.0, 0.8), // Bleu
            line_width: 1.0,
            dash_length: 5.0,
            gap_length: 3.0,
        }
    }
}

/// Rend une ligne horizontale pointillée au niveau du prix courant
pub fn render_current_price_line(
    frame: &mut Frame,
    viewport: &Viewport,
    current_price: f64,
    style: Option<CurrentPriceStyle>,
) {
    let style = style.unwrap_or_default();
    
    let y = viewport.price_scale().price_to_y(current_price);
    
    // Ne dessiner que si visible
    if y < 0.0 || y > viewport.height() {
        return;
    }
    
    // Dessiner une ligne pointillée
    let mut x = 0.0;
    let width = viewport.width();
    
    while x < width {
        let end_x = (x + style.dash_length).min(width);
        
        let dash = Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(end_x, y));
        });
        
        let stroke = Stroke::default()
            .with_color(style.line_color)
            .with_width(style.line_width);
        
        frame.stroke(&dash, stroke);
        
        x += style.dash_length + style.gap_length;
    }
}

