//! Rendu des lignes horizontales

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point, Size};

use crate::finance_chart::tools_canvas::DrawnHorizontalLine;
use crate::finance_chart::viewport::Viewport;

/// Dessine une ligne horizontale
pub fn draw_horizontal_line(
    frame: &mut Frame,
    viewport: &Viewport,
    line: &DrawnHorizontalLine,
    is_selected: bool,
) {
    let price_scale = viewport.price_scale();
    let y = price_scale.price_to_y(line.price);

    // Ne pas dessiner si hors de vue
    if y < -10.0 || y > viewport.height() + 10.0 {
        return;
    }

    let width = viewport.width();

    if line.dashed {
        // Ligne pointillée
        draw_dashed_line(frame, y, width, line.color, line.width, is_selected);
    } else {
        // Ligne pleine
        let path = Path::new(|builder| {
            builder.move_to(Point::new(0.0, y));
            builder.line_to(Point::new(width, y));
        });
        
        let line_color = if is_selected {
            Color::WHITE
        } else {
            line.color
        };
        
        let stroke = Stroke::default()
            .with_color(line_color)
            .with_width(if is_selected { line.width + 1.0 } else { line.width });
        frame.stroke(&path, stroke);
    }

    // Badge de prix si sélectionné
    if is_selected {
        draw_price_badge(frame, y, width, line.price, line.color);
    }
}

/// Dessine une ligne pointillée
fn draw_dashed_line(
    frame: &mut Frame,
    y: f32,
    width: f32,
    color: Color,
    line_width: f32,
    is_selected: bool,
) {
    let dash_length = 8.0;
    let gap_length = 4.0;
    
    let line_color = if is_selected {
        Color::WHITE
    } else {
        color
    };
    let stroke_width = if is_selected { line_width + 1.0 } else { line_width };

    let mut x = 0.0;
    while x < width {
        let end_x = (x + dash_length).min(width);
        let dash = Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(end_x, y));
        });
        let stroke = Stroke::default()
            .with_color(line_color)
            .with_width(stroke_width);
        frame.stroke(&dash, stroke);
        x += dash_length + gap_length;
    }
}

/// Dessine un badge de prix sur le bord droit
fn draw_price_badge(frame: &mut Frame, y: f32, width: f32, price: f64, color: Color) {
    let badge_width = 60.0;
    let badge_height = 16.0;
    let badge_x = width - badge_width - 5.0;
    let badge_y = y - badge_height / 2.0;

    // Fond du badge
    let bg_rect = Path::rectangle(
        Point::new(badge_x, badge_y),
        Size::new(badge_width, badge_height),
    );
    frame.fill(&bg_rect, color);

    // Texte du prix
    let price_str = format_price(price);
    let text = Text {
        content: price_str,
        position: Point::new(badge_x + 4.0, badge_y + 2.0),
        color: Color::BLACK,
        size: iced::Pixels(11.0),
        ..Text::default()
    };
    frame.fill_text(text);
}

/// Formate un prix pour l'affichage
fn format_price(price: f64) -> String {
    if price >= 10000.0 {
        format!("{:.0}", price)
    } else if price >= 100.0 {
        format!("{:.1}", price)
    } else {
        format!("{:.2}", price)
    }
}

/// Dessine l'aperçu d'une ligne horizontale en cours de création
pub fn draw_hline_preview(frame: &mut Frame, y: f32, width: f32) {
    let dash_length = 8.0;
    let gap_length = 4.0;
    let color = Color::from_rgba(1.0, 0.8, 0.0, 0.6);

    let mut x = 0.0;
    while x < width {
        let end_x = (x + dash_length).min(width);
        let dash = Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(end_x, y));
        });
        let stroke = Stroke::default()
            .with_color(color)
            .with_width(1.5);
        frame.stroke(&dash, stroke);
        x += dash_length + gap_length;
    }
}

/// Hit-test pour les lignes horizontales
pub fn hit_test_hline(
    mouse_y: f32,
    lines: &[DrawnHorizontalLine],
    viewport: &Viewport,
) -> Option<usize> {
    const HIT_TOLERANCE: f32 = 5.0;
    let price_scale = viewport.price_scale();

    for (index, line) in lines.iter().enumerate().rev() {
        let y = price_scale.price_to_y(line.price);
        if (mouse_y - y).abs() <= HIT_TOLERANCE {
            return Some(index);
        }
    }

    None
}

