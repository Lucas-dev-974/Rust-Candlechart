//! Module de configuration et settings du graphique

use iced::Color;
use serde::{Deserialize, Serialize};

/// Style personnalisable du graphique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartStyle {
    /// Couleur de fond du graphique
    pub background_color: SerializableColor,
    /// Couleur des bougies haussières
    pub bullish_color: SerializableColor,
    /// Couleur des bougies baissières
    pub bearish_color: SerializableColor,
    /// Couleur des mèches
    pub wick_color: SerializableColor,
    /// Couleur de la grille
    pub grid_color: SerializableColor,
    /// Couleur de la ligne de prix courant
    pub current_price_color: SerializableColor,
    /// Couleur du crosshair
    pub crosshair_color: SerializableColor,
    /// Couleur du texte
    pub text_color: SerializableColor,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background_color: SerializableColor::from_rgb(0.06, 0.06, 0.08),
            bullish_color: SerializableColor::from_rgb(0.0, 0.8, 0.0),
            bearish_color: SerializableColor::from_rgb(0.8, 0.0, 0.0),
            wick_color: SerializableColor::from_rgb(0.5, 0.5, 0.5),
            grid_color: SerializableColor::from_rgba(0.5, 0.5, 0.5, 0.3),
            current_price_color: SerializableColor::from_rgba(0.2, 0.6, 1.0, 0.8),
            crosshair_color: SerializableColor::from_rgba(0.6, 0.6, 0.6, 0.8),
            text_color: SerializableColor::from_rgba(0.8, 0.8, 0.8, 1.0),
        }
    }
}

impl ChartStyle {
    /// Sauvegarde les settings dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Charge les settings depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let style: ChartStyle = serde_json::from_str(&json)?;
        Ok(style)
    }
}

/// Couleur sérialisable (wrapper autour de iced::Color)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl SerializableColor {
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_iced(color: Color) -> Self {
        Self { r: color.r, g: color.g, b: color.b, a: color.a }
    }

    pub fn to_iced(self) -> Color {
        Color::from_rgba(self.r, self.g, self.b, self.a)
    }

}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}


/// État du dialog settings
#[derive(Debug, Clone, Default)]
pub struct SettingsState {
    /// Le dialog est-il ouvert
    pub is_open: bool,
}

impl SettingsState {
}

/// Définition d'un champ de couleur éditable
pub struct ColorField {
    pub label: &'static str,
    pub get: fn(&ChartStyle) -> SerializableColor,
    pub set: fn(&mut ChartStyle, SerializableColor),
}

/// Liste des champs de couleur éditables
pub fn color_fields() -> Vec<ColorField> {
    vec![
        ColorField {
            label: "Fond",
            get: |s| s.background_color,
            set: |s, c| s.background_color = c,
        },
        ColorField {
            label: "Bougie Haussière",
            get: |s| s.bullish_color,
            set: |s, c| s.bullish_color = c,
        },
        ColorField {
            label: "Bougie Baissière",
            get: |s| s.bearish_color,
            set: |s, c| s.bearish_color = c,
        },
        ColorField {
            label: "Mèches",
            get: |s| s.wick_color,
            set: |s, c| s.wick_color = c,
        },
        ColorField {
            label: "Grille",
            get: |s| s.grid_color,
            set: |s, c| s.grid_color = c,
        },
        ColorField {
            label: "Prix Courant",
            get: |s| s.current_price_color,
            set: |s, c| s.current_price_color = c,
        },
        ColorField {
            label: "Crosshair",
            get: |s| s.crosshair_color,
            set: |s, c| s.crosshair_color = c,
        },
        ColorField {
            label: "Texte",
            get: |s| s.text_color,
            set: |s, c| s.text_color = c,
        },
    ]
}

/// Couleurs prédéfinies pour le sélecteur
pub fn preset_colors() -> Vec<SerializableColor> {
    vec![
        // Verts
        SerializableColor::from_rgb(0.0, 0.8, 0.0),
        SerializableColor::from_rgb(0.0, 0.6, 0.0),
        SerializableColor::from_rgb(0.2, 0.8, 0.4),
        SerializableColor::from_rgb(0.0, 1.0, 0.5),
        // Rouges
        SerializableColor::from_rgb(0.8, 0.0, 0.0),
        SerializableColor::from_rgb(0.6, 0.0, 0.0),
        SerializableColor::from_rgb(1.0, 0.2, 0.2),
        SerializableColor::from_rgb(0.8, 0.2, 0.4),
        // Bleus
        SerializableColor::from_rgb(0.2, 0.6, 1.0),
        SerializableColor::from_rgb(0.0, 0.4, 0.8),
        SerializableColor::from_rgb(0.4, 0.6, 0.9),
        SerializableColor::from_rgb(0.0, 0.8, 1.0),
        // Gris
        SerializableColor::from_rgb(0.3, 0.3, 0.3),
        SerializableColor::from_rgb(0.5, 0.5, 0.5),
        SerializableColor::from_rgb(0.7, 0.7, 0.7),
        SerializableColor::from_rgb(0.9, 0.9, 0.9),
        // Fond sombres
        SerializableColor::from_rgb(0.06, 0.06, 0.08),
        SerializableColor::from_rgb(0.1, 0.1, 0.12),
        SerializableColor::from_rgb(0.12, 0.12, 0.15),
        SerializableColor::from_rgb(0.15, 0.15, 0.18),
        // Jaune/Orange
        SerializableColor::from_rgb(1.0, 0.8, 0.0),
        SerializableColor::from_rgb(1.0, 0.6, 0.0),
        SerializableColor::from_rgb(1.0, 0.4, 0.0),
        SerializableColor::from_rgb(0.8, 0.6, 0.2),
    ]
}

