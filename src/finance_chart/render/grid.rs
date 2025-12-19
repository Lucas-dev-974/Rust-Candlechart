use iced::widget::canvas::{self, Frame, Path};
use iced::{Color, Point};
use chrono::{DateTime, Utc, TimeZone};

use super::super::viewport::Viewport;

/// Couleurs par défaut pour la grille
pub struct GridStyle {
    pub line_color: Color,
    pub line_width: f32,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            line_color: Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            line_width: 1.0,
        }
    }
}

/// Calcule un pas "rond" approprié pour la plage donnée
/// afin d'avoir environ 5-10 lignes de grille
pub fn calculate_nice_step(range: f64) -> f64 {
    if range <= 0.0 {
        return 1.0;
    }
    
    // Trouver l'ordre de grandeur
    let raw_step = range / 6.0; // Viser ~6 lignes
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    let normalized = raw_step / magnitude;
    
    // Choisir un pas "rond" : 1, 2, 5, 10
    let nice_step = if normalized <= 1.5 {
        1.0
    } else if normalized <= 3.0 {
        2.0
    } else if normalized <= 7.0 {
        5.0
    } else {
        10.0
    };
    
    nice_step * magnitude
}

/// Calcule un pas temporel "rond" approprié (en secondes)
pub fn calculate_nice_time_step(range_seconds: i64) -> i64 {
    if range_seconds <= 0 {
        return 3600; // 1 heure par défaut
    }
    
    // Pas possibles en secondes
    const STEPS: &[i64] = &[
        60,           // 1 minute
        300,          // 5 minutes
        600,          // 10 minutes
        1800,         // 30 minutes
        3600,         // 1 heure
        7200,         // 2 heures
        14400,        // 4 heures
        21600,        // 6 heures
        43200,        // 12 heures
        86400,        // 1 jour
        172800,       // 2 jours
        604800,       // 1 semaine
        2592000,      // ~1 mois (30 jours)
    ];
    
    let target_step = range_seconds / 6; // Viser ~6 lignes
    
    // Trouver le pas le plus proche
    for &step in STEPS {
        if step >= target_step {
            return step;
        }
    }
    
    // Si la plage est très grande, utiliser des multiples de mois
    let months = (target_step / 2592000).max(1);
    months * 2592000
}

/// Formate un timestamp en chaîne lisible selon le contexte
pub fn format_time(timestamp: i64, step: i64) -> String {
    // Convertir le timestamp Unix (secondes) en DateTime
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    
    if step < 3600 {
        // Moins d'une heure : afficher HH:MM
        datetime.format("%H:%M").to_string()
    } else if step < 86400 {
        // Moins d'un jour : afficher HH:00
        datetime.format("%H:00").to_string()
    } else if step < 2592000 {
        // Moins d'un mois : afficher dd/mm
        datetime.format("%d/%m").to_string()
    } else {
        // Plus d'un mois : afficher dd/mm/yy
        datetime.format("%d/%m/%y").to_string()
    }
}

/// Rend une grille sur le frame (lignes uniquement, sans labels)
pub fn render_grid(frame: &mut Frame, viewport: &Viewport, style: Option<GridStyle>) {
    let style = style.unwrap_or_default();
    
    // === Lignes horizontales (niveaux de prix ronds) ===
    let (min_price, max_price) = viewport.price_scale().price_range();
    let price_range = max_price - min_price;
    let price_step = calculate_nice_step(price_range);
    
    // Trouver le premier niveau rond >= min_price
    let first_price = (min_price / price_step).ceil() * price_step;
    
    let mut price = first_price;
    while price <= max_price {
        let y = viewport.price_scale().price_to_y(price);
        
        // Ne dessiner que si visible
        if y >= 0.0 && y <= viewport.height() {
            let line = Path::new(|builder| {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(viewport.width(), y));
            });
            let stroke = canvas::Stroke::default()
                .with_color(style.line_color)
                .with_width(style.line_width);
            frame.stroke(&line, stroke);
        }
        
        price += price_step;
    }

    // === Lignes verticales (timestamps ronds) ===
    let (min_time, max_time) = viewport.time_scale().time_range();
    let time_range = max_time - min_time;
    let time_step = calculate_nice_time_step(time_range);
    
    // Trouver le premier timestamp rond >= min_time
    let first_time = ((min_time / time_step) + 1) * time_step;
    
    let mut time = first_time;
    while time <= max_time {
        let x = viewport.time_scale().time_to_x(time);
        
        // Ne dessiner que si visible
        if x >= 0.0 && x <= viewport.width() {
            let line = Path::new(|builder| {
                builder.move_to(Point::new(x, 0.0));
                builder.line_to(Point::new(x, viewport.height()));
            });
            let stroke = canvas::Stroke::default()
                .with_color(style.line_color)
                .with_width(style.line_width);
            frame.stroke(&line, stroke);
        }
        
        time += time_step;
    }
}

