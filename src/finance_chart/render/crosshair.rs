//! Rendu du crosshair (réticule) avec affichage prix/date

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point, Size};

use crate::finance_chart::viewport::Viewport;
use crate::finance_chart::render::utils::format_price_compact;

/// Style du crosshair
#[derive(Clone)]
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
/// Affiche uniquement la ligne horizontale et les labels
/// (La ligne verticale est gérée par le composant overlay)
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

    // === Ligne horizontale (prix) ===
    let horizontal_line = Path::new(|builder| {
        builder.move_to(Point::new(0.0, mouse_position.y));
        builder.line_to(Point::new(width, mouse_position.y));
    });
    let stroke = Stroke::default()
        .with_color(style.line_color)
        .with_width(style.line_width);
    frame.stroke(&horizontal_line, stroke);

    // === Label du prix (sur le bord droit) ===
    let price = viewport.price_scale().y_to_price(mouse_position.y);
    let price_label = format_price_compact(price);
    draw_price_label(frame, &style, mouse_position.y, width, &price_label);

    // Note: Le label du temps est géré par le composant overlay
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
pub fn draw_time_label(frame: &mut Frame, style: &CrosshairStyle, x: f32, height: f32, label: &str) {
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

/// Dessine le crosshair pour le graphique de volume
/// Affiche uniquement la ligne horizontale et le label du volume à la position Y de la souris
/// (La ligne verticale est gérée par le composant overlay)
pub fn render_volume_crosshair(
    frame: &mut Frame,
    _main_viewport: &Viewport,
    _main_mouse_position: Option<Point>,
    volume_scale: &crate::finance_chart::scale::VolumeScale,
    chart_bounds_width: f32,
    chart_bounds_height: f32,
    mouse_y_in_chart: Option<f32>,
    style: Option<CrosshairStyle>,
    _mouse_x_in_chart: Option<f32>,
) {
    // Si la souris est dans ce graphique, afficher la ligne horizontale et le label du volume
    if let Some(y) = mouse_y_in_chart {
        if y >= 0.0 && y <= chart_bounds_height {
            let style = style.unwrap_or_default();
            
            // Ligne horizontale
            let horizontal_line = Path::new(|builder| {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(chart_bounds_width, y));
            });
            let stroke = Stroke::default()
                .with_color(style.line_color)
                .with_width(style.line_width);
            frame.stroke(&horizontal_line, stroke);
            
            // Label du volume (sur le bord droit)
            let volume = volume_scale.y_to_volume(y);
            let volume_label = format!("{:.0}", volume);
            draw_price_label(frame, &style, y, chart_bounds_width, &volume_label);
        }
    }
}

/// Dessine le crosshair pour le graphique RSI
/// Affiche uniquement la ligne horizontale et le label du RSI à la position Y de la souris
/// (La ligne verticale est gérée par le composant overlay)
pub fn render_rsi_crosshair(
    frame: &mut Frame,
    _main_viewport: &Viewport,
    _main_mouse_position: Option<Point>,
    chart_bounds_width: f32,
    chart_bounds_height: f32,
    mouse_y_in_chart: Option<f32>,
    style: Option<CrosshairStyle>,
    _mouse_x_in_chart: Option<f32>,
) {
    // Si la souris est dans ce graphique, afficher la ligne horizontale et le label du RSI
    if let Some(y) = mouse_y_in_chart {
        if y >= 0.0 && y <= chart_bounds_height {
            let style = style.unwrap_or_default();
            
            // Ligne horizontale
            let horizontal_line = Path::new(|builder| {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(chart_bounds_width, y));
            });
            let stroke = Stroke::default()
                .with_color(style.line_color)
                .with_width(style.line_width);
            frame.stroke(&horizontal_line, stroke);
            
            // Label du RSI (sur le bord droit)
            // RSI va de 0 (bas) à 100 (haut)
            let normalized_rsi = 1.0 - (y / chart_bounds_height);
            let rsi_value = (normalized_rsi * 100.0).clamp(0.0, 100.0);
            let rsi_label = format!("{:.1}", rsi_value);
            draw_price_label(frame, &style, y, chart_bounds_width, &rsi_label);
        }
    }
}

/// Dessine le crosshair pour le graphique MACD
/// Affiche uniquement la ligne horizontale et le label du MACD à la position Y de la souris
/// (La ligne verticale est gérée par le composant overlay)
pub fn render_macd_crosshair(
    frame: &mut Frame,
    _main_viewport: &Viewport,
    _main_mouse_position: Option<Point>,
    chart_bounds_width: f32,
    chart_bounds_height: f32,
    mouse_y_in_chart: Option<f32>,
    y_to_macd: &dyn Fn(f32) -> f64,
    style: Option<CrosshairStyle>,
    _mouse_x_in_chart: Option<f32>,
) {
    // Si la souris est dans ce graphique, afficher la ligne horizontale et le label du MACD
    if let Some(y) = mouse_y_in_chart {
        if y >= 0.0 && y <= chart_bounds_height {
            let style = style.unwrap_or_default();
            
            // Ligne horizontale
            let horizontal_line = Path::new(|builder| {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(chart_bounds_width, y));
            });
            let stroke = Stroke::default()
                .with_color(style.line_color)
                .with_width(style.line_width);
            frame.stroke(&horizontal_line, stroke);
            
            // Label du MACD (sur le bord droit)
            let macd_value = y_to_macd(y);
            let macd_label = if macd_value.abs() >= 1.0 {
                format!("{:.2}", macd_value)
            } else {
                format!("{:.4}", macd_value)
            };
            draw_price_label(frame, &style, y, chart_bounds_width, &macd_label);
        }
    }
}

