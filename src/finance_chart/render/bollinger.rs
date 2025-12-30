//! Rendu des bandes de Bollinger sur le graphique principal

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point};

use crate::finance_chart::viewport::Viewport;
use crate::finance_chart::core::Candle;
use crate::finance_chart::indicators::bollinger::BollingerValue;

/// Style pour les bandes de Bollinger
pub struct BollingerStyle {
    pub middle_color: Color,      // Couleur de la bande moyenne
    pub upper_color: Color,        // Couleur de la bande supérieure
    pub lower_color: Color,        // Couleur de la bande inférieure
    pub fill_color: Color,         // Couleur de remplissage entre les bandes
    pub line_width: f32,           // Épaisseur des lignes
}

impl Default for BollingerStyle {
    fn default() -> Self {
        Self {
            middle_color: Color::from_rgba(1.0, 1.0, 0.0, 0.8),      // Jaune pour la moyenne
            upper_color: Color::from_rgba(0.3, 0.7, 1.0, 0.8),       // Bleu clair pour la supérieure
            lower_color: Color::from_rgba(0.3, 0.7, 1.0, 0.8),       // Bleu clair pour l'inférieure
            fill_color: Color::from_rgba(0.3, 0.7, 1.0, 0.15),      // Bleu très transparent pour le remplissage
            line_width: 1.5,
        }
    }
}

/// Rend les bandes de Bollinger sur le graphique principal
/// 
/// # Arguments
/// * `frame` - Frame de rendu Iced
/// * `viewport` - Viewport pour les conversions de coordonnées
/// * `candles` - Bougies visibles sur le graphique
/// * `bollinger_values` - Valeurs des bandes de Bollinger pré-calculées correspondant aux bougies visibles
/// * `style` - Style optionnel pour personnaliser les couleurs
pub fn render_bollinger_bands(
    frame: &mut Frame,
    viewport: &Viewport,
    candles: &[Candle],
    bollinger_values: &[Option<BollingerValue>],
    style: Option<BollingerStyle>,
) {
    if candles.is_empty() || bollinger_values.is_empty() {
        return;
    }
    
    // S'assurer que les deux slices ont la même longueur
    let min_len = candles.len().min(bollinger_values.len());
    let candles = &candles[..min_len];
    let bollinger_values = &bollinger_values[..min_len];
    
    let style = style.unwrap_or_default();
    
    // Filtrer les valeurs valides et les convertir en points
    let mut middle_points = Vec::new();
    let mut upper_points = Vec::new();
    let mut lower_points = Vec::new();
    
    for (candle, bb_opt) in candles.iter().zip(bollinger_values.iter()) {
        if let Some(bb) = bb_opt {
            let x = viewport.time_scale().time_to_x(candle.timestamp);
            
            // Ne garder que les points visibles
            if x >= -10.0 && x <= viewport.width() + 10.0 {
                let middle_y = viewport.price_scale().price_to_y(bb.middle);
                let upper_y = viewport.price_scale().price_to_y(bb.upper);
                let lower_y = viewport.price_scale().price_to_y(bb.lower);
                
                middle_points.push(Point::new(x, middle_y));
                upper_points.push(Point::new(x, upper_y));
                lower_points.push(Point::new(x, lower_y));
            }
        }
    }
    
    if middle_points.is_empty() {
        return;
    }
    
    // Dessiner la zone de remplissage entre les bandes supérieure et inférieure
    if middle_points.len() > 1 {
        let fill_path = Path::new(|builder| {
            // Commencer par la bande supérieure (de gauche à droite)
            if let Some(first) = upper_points.first() {
                builder.move_to(*first);
            }
            for point in &upper_points[1..] {
                builder.line_to(*point);
            }
            
            // Puis la bande inférieure (de droite à gauche)
            if let Some(last) = lower_points.last() {
                builder.line_to(*last);
            }
            for point in lower_points.iter().rev().skip(1) {
                builder.line_to(*point);
            }
            
            // Fermer le chemin
            if let Some(first) = upper_points.first() {
                builder.line_to(*first);
            }
        });
        
        frame.fill(&fill_path, style.fill_color);
    }
    
    // Dessiner les trois lignes (moyenne, supérieure, inférieure)
    if middle_points.len() > 1 {
        // Ligne moyenne
        let middle_path = Path::new(|builder| {
            if let Some(first) = middle_points.first() {
                builder.move_to(*first);
            }
            for point in &middle_points[1..] {
                builder.line_to(*point);
            }
        });
        let middle_stroke = Stroke::default()
            .with_color(style.middle_color)
            .with_width(style.line_width);
        frame.stroke(&middle_path, middle_stroke);
        
        // Ligne supérieure
        let upper_path = Path::new(|builder| {
            if let Some(first) = upper_points.first() {
                builder.move_to(*first);
            }
            for point in &upper_points[1..] {
                builder.line_to(*point);
            }
        });
        let upper_stroke = Stroke::default()
            .with_color(style.upper_color)
            .with_width(style.line_width);
        frame.stroke(&upper_path, upper_stroke);
        
        // Ligne inférieure
        let lower_path = Path::new(|builder| {
            if let Some(first) = lower_points.first() {
                builder.move_to(*first);
            }
            for point in &lower_points[1..] {
                builder.line_to(*point);
            }
        });
        let lower_stroke = Stroke::default()
            .with_color(style.lower_color)
            .with_width(style.line_width);
        frame.stroke(&lower_path, lower_stroke);
    }
}

