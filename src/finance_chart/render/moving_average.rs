//! Rendu de la moyenne mobile sur le graphique principal

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point};

use crate::finance_chart::viewport::Viewport;
use crate::finance_chart::core::Candle;

/// Style pour la moyenne mobile
pub struct MovingAverageStyle {
    pub color: Color,        // Couleur de la ligne
    pub line_width: f32,     // Épaisseur de la ligne
}

impl Default for MovingAverageStyle {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(1.0, 0.5, 0.0, 0.9),  // Orange pour la MA
            line_width: 2.0,
        }
    }
}

/// Rend la moyenne mobile sur le graphique principal
/// 
/// # Arguments
/// * `frame` - Frame de rendu Iced
/// * `viewport` - Viewport pour les conversions de coordonnées
/// * `candles` - Bougies visibles sur le graphique
/// * `ma_values` - Valeurs de la moyenne mobile pré-calculées correspondant aux bougies visibles
/// * `style` - Style optionnel pour personnaliser la couleur et l'épaisseur
pub fn render_moving_average(
    frame: &mut Frame,
    viewport: &Viewport,
    candles: &[Candle],
    ma_values: &[Option<f64>],
    style: Option<MovingAverageStyle>,
) {
    if candles.is_empty() || ma_values.is_empty() {
        return;
    }
    
    // S'assurer que les deux slices ont la même longueur
    let min_len = candles.len().min(ma_values.len());
    let candles = &candles[..min_len];
    let ma_values = &ma_values[..min_len];
    
    let style = style.unwrap_or_default();
    
    // Filtrer les valeurs valides et les convertir en points
    let mut ma_points = Vec::new();
    
    for (candle, ma_opt) in candles.iter().zip(ma_values.iter()) {
        if let Some(ma_value) = ma_opt {
            let x = viewport.time_scale().time_to_x(candle.timestamp);
            
            // Ne garder que les points visibles
            if x >= -10.0 && x <= viewport.width() + 10.0 {
                let y = viewport.price_scale().price_to_y(*ma_value);
                ma_points.push(Point::new(x, y));
            }
        }
    }
    
    if ma_points.len() < 2 {
        return;
    }
    
    // Dessiner la ligne de la moyenne mobile
    let ma_path = Path::new(|builder| {
        if let Some(first) = ma_points.first() {
            builder.move_to(*first);
        }
        for point in &ma_points[1..] {
            builder.line_to(*point);
        }
    });
    
    let ma_stroke = Stroke::default()
        .with_color(style.color)
        .with_width(style.line_width);
    frame.stroke(&ma_path, ma_stroke);
}

