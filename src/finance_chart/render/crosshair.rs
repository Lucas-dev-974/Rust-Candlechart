//! Rendu du crosshair (réticule) avec affichage prix/date

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point, Size};

use crate::finance_chart::viewport::Viewport;
use crate::finance_chart::render::grid::format_time;
use crate::finance_chart::render::utils::format_price_compact;

/// Style du crosshair
pub struct CrosshairStyle {
    pub line_color: Color,
    pub line_width: f32,
    pub label_bg_color: Color,
    pub label_text_color: Color,
    pub label_text_size: f32,
}

impl Default for CrosshairStyle {
    fn default() -> Self {
        Self {
            line_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            line_width: 1.0,
            label_bg_color: Color::from_rgba(0.2, 0.2, 0.25, 0.95),
            label_text_color: Color::WHITE,
            label_text_size: 11.0,
        }
    }
}

/// Dessine le crosshair à la position de la souris
pub fn render_crosshair(
    frame: &mut Frame,
    viewport: &Viewport,
    mouse_position: Point,
    style: Option<CrosshairStyle>,
) {
    let style = style.unwrap_or_default();
    let width = viewport.width();
    let height = viewport.height();

    // Ne pas dessiner si hors du viewport
    if mouse_position.x < 0.0 || mouse_position.x > width ||
       mouse_position.y < 0.0 || mouse_position.y > height {
        return;
    }

    // === Ligne verticale (temps) ===
    let vertical_line = Path::new(|builder| {
        builder.move_to(Point::new(mouse_position.x, 0.0));
        builder.line_to(Point::new(mouse_position.x, height));
    });
    let stroke = Stroke::default()
        .with_color(style.line_color)
        .with_width(style.line_width);
    frame.stroke(&vertical_line, stroke.clone());

    // === Ligne horizontale (prix) ===
    let horizontal_line = Path::new(|builder| {
        builder.move_to(Point::new(0.0, mouse_position.y));
        builder.line_to(Point::new(width, mouse_position.y));
    });
    frame.stroke(&horizontal_line, stroke);

    // === Label du prix (sur le bord droit) ===
    let price = viewport.price_scale().y_to_price(mouse_position.y);
    let price_label = format_price_compact(price);
    draw_price_label(frame, &style, mouse_position.y, width, &price_label);

    // === Label du temps (sur le bord bas) ===
    let timestamp = viewport.time_scale().x_to_time(mouse_position.x);
    let time_label = format_time(timestamp, 3600); // Format avec précision horaire
    draw_time_label(frame, &style, mouse_position.x, height, &time_label);
}

/// Dessine le label du prix sur le bord droit
fn draw_price_label(frame: &mut Frame, style: &CrosshairStyle, y: f32, width: f32, label: &str) {
    let padding_x = 4.0;
    let padding_y = 2.0;
    let label_width = 60.0;
    let label_height = style.label_text_size + padding_y * 2.0;
    
    let label_x = width - label_width - 2.0;
    let label_y = y - label_height / 2.0;

    // Fond du label
    let bg_rect = Path::rectangle(
        Point::new(label_x, label_y),
        Size::new(label_width, label_height),
    );
    frame.fill(&bg_rect, style.label_bg_color);

    // Texte
    let text = Text {
        content: label.to_string(),
        position: Point::new(label_x + padding_x, label_y + padding_y),
        color: style.label_text_color,
        size: iced::Pixels(style.label_text_size),
        ..Text::default()
    };
    frame.fill_text(text);
}

/// Dessine le label du temps sur le bord bas
fn draw_time_label(frame: &mut Frame, style: &CrosshairStyle, x: f32, height: f32, label: &str) {
    let padding_x = 4.0;
    let padding_y = 2.0;
    let label_width = 50.0;
    let label_height = style.label_text_size + padding_y * 2.0;
    
    let label_x = x - label_width / 2.0;
    let label_y = height - label_height - 2.0;

    // Fond du label
    let bg_rect = Path::rectangle(
        Point::new(label_x, label_y),
        Size::new(label_width, label_height),
    );
    frame.fill(&bg_rect, style.label_bg_color);

    // Texte
    let text = Text {
        content: label.to_string(),
        position: Point::new(label_x + padding_x, label_y + padding_y),
        color: style.label_text_color,
        size: iced::Pixels(style.label_text_size),
        ..Text::default()
    };
    frame.fill_text(text);
}

