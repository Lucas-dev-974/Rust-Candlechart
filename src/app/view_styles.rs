//! Styles réutilisables pour les vues de l'application
//!
//! Ce module contient des fonctions helper pour les styles de widgets communs
//! afin de réduire la duplication de code et assurer la cohérence visuelle.

use iced::widget::{button, container};
use iced::{Color, Background, Border};

/// Couleurs de thème de l'application
#[allow(dead_code)]
pub mod colors {
    use iced::Color;
    
    /// Fond principal sombre
    pub const BACKGROUND_DARK: Color = Color::from_rgb(0.08, 0.08, 0.10);
    /// Fond légèrement plus clair
    pub const BACKGROUND_MEDIUM: Color = Color::from_rgb(0.10, 0.10, 0.12);
    /// Fond pour les headers
    pub const BACKGROUND_HEADER: Color = Color::from_rgb(0.12, 0.12, 0.15);
    /// Bordure standard
    pub const BORDER_STANDARD: Color = Color::from_rgb(0.2, 0.2, 0.25);
    /// Bordure pour les éléments actifs
    pub const BORDER_ACTIVE: Color = Color::from_rgb(0.4, 0.6, 0.8);
    /// Bordure pour le séparateur
    pub const SEPARATOR: Color = Color::from_rgb(0.3, 0.3, 0.35);
    /// Texte principal
    pub const TEXT_PRIMARY: Color = Color::WHITE;
    /// Texte secondaire
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.7, 0.7, 0.7);
    /// Texte tertiaire
    pub const TEXT_TERTIARY: Color = Color::from_rgb(0.8, 0.8, 0.8);
    /// Couleur accent (hover)
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.2, 0.2, 0.25);
    /// Couleur bouton actif
    pub const BUTTON_ACTIVE: Color = Color::from_rgb(0.25, 0.4, 0.6);
    /// Couleur bouton actif hover
    pub const BUTTON_ACTIVE_HOVER: Color = Color::from_rgb(0.3, 0.5, 0.7);
    /// Couleur succès
    pub const SUCCESS: Color = Color::from_rgb(0.2, 0.5, 0.2);
    /// Couleur danger
    pub const DANGER: Color = Color::from_rgb(0.5, 0.2, 0.2);
    /// Couleur info
    pub const INFO: Color = Color::from_rgb(0.4, 0.8, 1.0);
}

/// Style de conteneur pour les panneaux
pub fn panel_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND_MEDIUM)),
        border: Border {
            color: colors::BORDER_STANDARD,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style de conteneur pour les panneaux sans bordure
pub fn panel_container_no_border_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND_MEDIUM)),
        ..Default::default()
    }
}

/// Style de conteneur pour les headers
pub fn header_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND_HEADER)),
        border: Border {
            color: colors::BORDER_STANDARD,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

/// Style de conteneur pour le fond principal
pub fn dark_background_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BACKGROUND_DARK)),
        ..Default::default()
    }
}

/// Style de bouton standard (icône/settings)
pub fn icon_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg_color = match status {
        button::Status::Hovered => colors::ACCENT_HOVER,
        _ => Color::from_rgb(0.15, 0.15, 0.18),
    };
    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color: colors::TEXT_PRIMARY,
        ..Default::default()
    }
}

/// Style de bouton pour les actions positives (appliquer, confirmer)
pub fn success_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(colors::SUCCESS)),
        text_color: colors::TEXT_PRIMARY,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style de bouton pour les actions négatives (annuler, supprimer)
pub fn danger_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg_color = match status {
        button::Status::Hovered => Color::from_rgb(0.3, 0.2, 0.2),
        _ => Color::from_rgb(0.25, 0.15, 0.15),
    };
    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color: colors::TEXT_PRIMARY,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Style pour le provider actif dans la liste
pub fn provider_card_style(is_active: bool) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.12))),
        border: Border {
            color: if is_active {
                colors::INFO
            } else {
                colors::BORDER_STANDARD
            },
            width: if is_active { 2.0 } else { 1.0 },
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Style pour le séparateur horizontal
#[allow(dead_code)]
pub fn separator_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::SEPARATOR)),
        ..Default::default()
    }
}

/// Style pour les boîtes de couleur
#[allow(dead_code)]
pub fn color_box_style(color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: colors::TEXT_PRIMARY,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

/// Style pour les presets de couleur
#[allow(dead_code)]
pub fn color_preset_style(color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

/// Style de bouton transparent (pour les color pickers)
#[allow(dead_code)]
pub fn transparent_button_style(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        ..Default::default()
    }
}

/// Style pour les conteneurs de presets
#[allow(dead_code)]
pub fn presets_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
        border: Border {
            color: Color::from_rgb(0.3, 0.3, 0.35),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    }
}

