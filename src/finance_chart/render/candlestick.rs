use iced::widget::canvas::{self, Frame, Path};
use iced::{Color, Point, Size};

use super::super::core::Candle;
use super::super::viewport::Viewport;
use super::bar_sizing::{calculate_bar_width, calculate_candle_period};

/// Couleurs par défaut pour les bougies
pub struct CandleColors {
    pub bullish: Color,
    pub bearish: Color,
    pub wick: Color,
}

impl Default for CandleColors {
    fn default() -> Self {
        Self {
            bullish: Color::from_rgb(0.0, 0.8, 0.0), // Vert
            bearish: Color::from_rgb(0.8, 0.0, 0.0), // Rouge
            wick: Color::from_rgb(0.5, 0.5, 0.5),   // Gris
        }
    }
}

/// Rend une bougie sur le frame
fn render_single_candle(
    frame: &mut Frame,
    candle: &Candle,
    viewport: &Viewport,
    candle_width: f32,
    colors: &CandleColors,
) {
    let price_scale = viewport.price_scale();
    let time_scale = viewport.time_scale();

    let x = time_scale.time_to_x(candle.timestamp);
    let open_y = price_scale.price_to_y(candle.open);
    let close_y = price_scale.price_to_y(candle.close);
    let high_y = price_scale.price_to_y(candle.high);
    let low_y = price_scale.price_to_y(candle.low);

    // Couleur selon si la bougie est haussière ou baissière
    let body_color = if candle.is_bullish() {
        colors.bullish
    } else {
        colors.bearish
    };

    // Dessiner la mèche (wick)
    let wick_path = Path::new(|builder| {
        builder.move_to(Point::new(x, high_y));
        builder.line_to(Point::new(x, low_y));
    });
    frame.stroke(&wick_path, canvas::Stroke::default().with_color(colors.wick).with_width(1.0));

    // Dessiner le body
    let body_top = open_y.min(close_y);
    let body_bottom = open_y.max(close_y);
    let body_height = (body_bottom - body_top).max(1.0); // Minimum 1px pour visibilité

    let body_path = Path::rectangle(
        Point::new(x - candle_width / 2.0, body_top),
        Size::new(candle_width, body_height),
    );
    frame.fill(&body_path, body_color);
}

/// Rend toutes les bougies visibles sur le frame
pub fn render_candlesticks(
    frame: &mut Frame,
    candles: &[Candle],
    viewport: &Viewport,
    colors: Option<CandleColors>,
) {
    if candles.is_empty() {
        return;
    }

    let colors = colors.unwrap_or_default();

    // Calculer la largeur des bougies via le module bar_sizing
    let candle_period = calculate_candle_period(candles);
    let (min_time, max_time) = viewport.time_scale().time_range();
    let candle_width = calculate_bar_width(candle_period, max_time - min_time, viewport.width());

    // Dessiner uniquement les bougies visibles
    // Pour les séries avec peu de bougies, dessiner toutes les bougies même si elles sont légèrement en dehors
    let is_small_series = candles.len() <= 50;
    let margin = if is_small_series { candle_width * 2.0 } else { candle_width };
    
    for candle in candles {
        // Vérifier si la bougie est visible horizontalement
        let x = viewport.time_scale().time_to_x(candle.timestamp);
        // Pour les petites séries, utiliser une marge plus large pour s'assurer que toutes les bougies sont visibles
        if x >= -margin && x <= viewport.width() + margin {
            render_single_candle(frame, candle, viewport, candle_width, &colors);
        }
    }
}

