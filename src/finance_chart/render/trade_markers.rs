//! Rendu des marqueurs de trades sur le graphique

use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point};
use crate::finance_chart::viewport::Viewport;
use crate::app::data::{Trade, TradeType};

/// Dessine un marqueur de trade sur le graphique
pub fn draw_trade_marker(
    frame: &mut Frame,
    viewport: &Viewport,
    trade: &Trade,
) {
    // Convertir le timestamp en position X
    let x = viewport.time_scale().time_to_x(trade.timestamp);
    
    // Convertir le prix en position Y
    let y = viewport.price_scale().price_to_y(trade.price);
    
    // Vérifier si le marqueur est visible dans le viewport
    let bounds = frame.size();
    if x < -20.0 || x > bounds.width + 20.0 || y < -20.0 || y > bounds.height + 20.0 {
        return;
    }
    
    // Couleur selon le type de trade
    let color = match trade.trade_type {
        TradeType::Buy => Color::from_rgb(0.0, 0.8, 0.0), // Vert pour achat
        TradeType::Sell => Color::from_rgb(0.8, 0.0, 0.0), // Rouge pour vente
    };
    
    // Taille du marqueur
    let marker_size = 8.0;
    
    // Dessiner un triangle pointant vers le haut pour achat (^)
    // ou vers le bas pour vente (v)
    let marker_path = match trade.trade_type {
        TradeType::Buy => {
            // Triangle pointant vers le haut (^)
            Path::new(|builder| {
                builder.move_to(Point::new(x, y - marker_size));
                builder.line_to(Point::new(x - marker_size, y));
                builder.line_to(Point::new(x + marker_size, y));
                builder.line_to(Point::new(x, y - marker_size));
            })
        }
        TradeType::Sell => {
            // Triangle pointant vers le bas (v)
            Path::new(|builder| {
                builder.move_to(Point::new(x, y + marker_size));
                builder.line_to(Point::new(x - marker_size, y));
                builder.line_to(Point::new(x + marker_size, y));
                builder.line_to(Point::new(x, y + marker_size));
            })
        }
    };
    
    // Remplir le triangle
    frame.fill(&marker_path, color);
    
    // Dessiner une bordure pour plus de visibilité
    let stroke = Stroke::default()
        .with_color(Color::from_rgb(0.1, 0.1, 0.1))
        .with_width(1.0);
    frame.stroke(&marker_path, stroke);
}

/// Dessine tous les marqueurs de trades visibles
pub fn render_trade_markers(
    frame: &mut Frame,
    viewport: &Viewport,
    trades: &[Trade],
    current_symbol: &str,
) {
    // Filtrer les trades pour le symbole actuel et qui sont dans le viewport
    let (min_time, max_time) = viewport.time_scale().time_range();
    
    for trade in trades {
        // Filtrer par symbole
        if trade.symbol != current_symbol {
            continue;
        }
        
        // Vérifier si le trade est dans la plage temporelle visible
        if trade.timestamp >= min_time && trade.timestamp <= max_time {
            draw_trade_marker(frame, viewport, trade);
        }
    }
}

