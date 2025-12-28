//! Style commun pour les axes du graphique et des indicateurs techniques

use iced::Color;

/// Style pour les axes (prix, temps, indicateurs)
pub struct AxisStyle {
    pub background_color: Color,
    pub text_color: Color,
    pub text_size: f32,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            background_color: Color::from_rgb(0.08, 0.08, 0.10),
            text_color: Color::from_rgb(0.7, 0.7, 0.7),
            text_size: 11.0,
        }
    }
}

