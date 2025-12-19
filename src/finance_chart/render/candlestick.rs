use iced::widget::canvas::{self, Frame, Path};
use iced::{Color, Point, Size};

use super::super::core::Candle;
use super::super::viewport::Viewport;

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

    // Constantes pour le dimensionnement des bougies
    const MIN_GAP: f32 = 3.0;      // Espacement minimum entre bougies
    const MAX_WIDTH: f32 = 20.0;   // Largeur maximum d'une bougie
    const MIN_WIDTH: f32 = 1.0;    // Largeur minimum d'une bougie

    // Détecter l'intervalle de temps entre les bougies (période)
    // Utilise les deux premières bougies pour déterminer la période
    let candle_period = if candles.len() >= 2 {
        (candles[1].timestamp - candles[0].timestamp).abs()
    } else {
        3600 // Par défaut 1 heure si une seule bougie
    };

    // Calculer l'espacement en pixels basé sur l'échelle temporelle (FIXE lors du pan)
    // C'est le nombre de pixels que représente une période de bougie
    let (min_time, max_time) = viewport.time_scale().time_range();
    let time_range = (max_time - min_time) as f64;
    let pixels_per_second = viewport.width() as f64 / time_range;
    let spacing = (candle_period as f64 * pixels_per_second) as f32;
    
    // Ratio adaptatif : plus il y a de bougies, plus le ratio diminue
    // Cela évite que les bougies s'agglutinent en zoom out
    let width_ratio = if spacing > 25.0 {
        1.0  // Très zoom in : bougies pleine largeur (100%)
    } else if spacing > 15.0 {
        0.8  // Zoom in : bougies larges (80%)
    } else if spacing > 8.0 {
        0.6  // Zoom moyen : bougies moyennes (60%)
    } else if spacing > 5.0 {
        0.4  // Zoom out : bougies fines (40%)
    } else {
        0.3  // Zoom très out : bougies très fines (30%)
    };
    
    // Calcul final : ratio adaptatif, gap minimum garanti, dans les limites min/max
    let candle_width = (spacing * width_ratio)
        .min(spacing - MIN_GAP)
        .clamp(MIN_WIDTH, MAX_WIDTH);

    // Dessiner uniquement les bougies visibles
    for candle in candles {
        // Vérifier si la bougie est visible horizontalement
        let x = viewport.time_scale().time_to_x(candle.timestamp);
        if x >= -candle_width && x <= viewport.width() + candle_width {
            render_single_candle(frame, candle, viewport, candle_width, &colors);
        }
    }
}

