//! Rendu du tooltip OHLC au survol des bougies

use iced::widget::canvas::{Frame, Path, Text};
use iced::{Color, Point, Size};
use chrono::{DateTime, Utc, TimeZone};

use crate::finance_chart::core::Candle;
use crate::finance_chart::viewport::Viewport;

/// Style du tooltip
pub struct TooltipStyle {
    pub bg_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub bullish_color: Color,
    pub bearish_color: Color,
    pub text_size: f32,
    pub padding: f32,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            bg_color: Color::from_rgba(0.1, 0.1, 0.12, 0.95),
            border_color: Color::from_rgba(0.3, 0.3, 0.35, 1.0),
            text_color: Color::from_rgba(0.8, 0.8, 0.8, 1.0),
            bullish_color: Color::from_rgb(0.0, 0.8, 0.0),
            bearish_color: Color::from_rgb(0.8, 0.0, 0.0),
            text_size: 11.0,
            padding: 8.0,
        }
    }
}

/// Trouve la bougie sous le curseur
pub fn find_candle_at_position<'a>(
    mouse_x: f32,
    candles: &'a [Candle],
    viewport: &Viewport,
) -> Option<&'a Candle> {
    if candles.is_empty() {
        return None;
    }

    let time_scale = viewport.time_scale();
    let mouse_time = time_scale.x_to_time(mouse_x);

    // Trouver l'intervalle entre bougies
    let candle_interval = if candles.len() >= 2 {
        (candles[1].timestamp - candles[0].timestamp).abs()
    } else {
        3600
    };

    // Chercher la bougie la plus proche
    let half_interval = candle_interval / 2;
    
    candles.iter().find(|c| {
        (c.timestamp - mouse_time).abs() <= half_interval
    })
}

/// Dessine le tooltip OHLC
pub fn render_tooltip(
    frame: &mut Frame,
    candle: &Candle,
    mouse_position: Point,
    viewport: &Viewport,
    style: Option<TooltipStyle>,
) {
    let style = style.unwrap_or_default();
    
    let line_height = style.text_size + 4.0;
    let tooltip_width = 140.0;
    let tooltip_height = line_height * 6.0 + style.padding * 2.0;

    // Positionner le tooltip (éviter de sortir du viewport)
    let mut tooltip_x = mouse_position.x + 15.0;
    let mut tooltip_y = mouse_position.y - tooltip_height / 2.0;

    if tooltip_x + tooltip_width > viewport.width() {
        tooltip_x = mouse_position.x - tooltip_width - 15.0;
    }
    if tooltip_y < 0.0 {
        tooltip_y = 0.0;
    }
    if tooltip_y + tooltip_height > viewport.height() {
        tooltip_y = viewport.height() - tooltip_height;
    }

    // Fond du tooltip
    let bg_rect = Path::rectangle(
        Point::new(tooltip_x, tooltip_y),
        Size::new(tooltip_width, tooltip_height),
    );
    frame.fill(&bg_rect, style.bg_color);

    // Bordure
    let border_stroke = iced::widget::canvas::Stroke::default()
        .with_color(style.border_color)
        .with_width(1.0);
    frame.stroke(&bg_rect, border_stroke);

    // Contenu
    let mut y = tooltip_y + style.padding;
    let x = tooltip_x + style.padding;

    // Date/Heure
    let datetime: DateTime<Utc> = Utc.timestamp_opt(candle.timestamp, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    let date_str = datetime.format("%d/%m/%Y %H:%M").to_string();
    draw_text_line(frame, &date_str, x, y, style.text_color, style.text_size);
    y += line_height;

    // Open
    let open_str = format!("O: {}", format_price(candle.open));
    draw_text_line(frame, &open_str, x, y, style.text_color, style.text_size);
    y += line_height;

    // High
    let high_str = format!("H: {}", format_price(candle.high));
    draw_text_line(frame, &high_str, x, y, style.text_color, style.text_size);
    y += line_height;

    // Low
    let low_str = format!("L: {}", format_price(candle.low));
    draw_text_line(frame, &low_str, x, y, style.text_color, style.text_size);
    y += line_height;

    // Close
    let close_str = format!("C: {}", format_price(candle.close));
    draw_text_line(frame, &close_str, x, y, style.text_color, style.text_size);
    y += line_height;

    // Change %
    let change_pct = ((candle.close - candle.open) / candle.open) * 100.0;
    let change_str = format!("{:+.2}%", change_pct);
    let change_color = if candle.is_bullish() {
        style.bullish_color
    } else {
        style.bearish_color
    };
    draw_text_line(frame, &change_str, x, y, change_color, style.text_size);
}

/// Formate un prix pour l'affichage
fn format_price(price: f64) -> String {
    if price >= 100.0 {
        format!("{:.2}", price)
    } else if price >= 1.0 {
        format!("{:.4}", price)
    } else {
        format!("{:.6}", price)
    }
}

/// Dessine une ligne de texte
fn draw_text_line(frame: &mut Frame, content: &str, x: f32, y: f32, color: Color, size: f32) {
    let text = Text {
        content: content.to_string(),
        position: Point::new(x, y),
        color,
        size: iced::Pixels(size),
        ..Text::default()
    };
    frame.fill_text(text);
}

