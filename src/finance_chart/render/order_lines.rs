//! Rendu des lignes d'ordres limit et TP/SL

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point, Size};

use crate::finance_chart::viewport::Viewport;
use crate::app::data::{PendingOrder, Position, TradeType};

/// Dessine les lignes des ordres limit en attente
pub fn draw_pending_order_lines(
    frame: &mut Frame,
    viewport: &Viewport,
    pending_orders: &[PendingOrder],
    current_symbol: &str,
) {
    for order in pending_orders {
        if order.symbol != current_symbol {
            continue;
        }
        
        let y = viewport.price_scale().price_to_y(order.limit_price);
        
        // Ne pas dessiner si hors de vue
        if y < -10.0 || y > viewport.height() + 10.0 {
            continue;
        }
        
        let width = viewport.width();
        
        // Couleur selon le type d'ordre (vert pour achat, rouge pour vente)
        let color = match order.trade_type {
            TradeType::Buy => Color::from_rgba(0.0, 0.8, 0.0, 0.7), // Vert
            TradeType::Sell => Color::from_rgba(0.8, 0.0, 0.0, 0.7), // Rouge
        };
        
        // Dessiner une ligne pointillée pour les ordres limit
        draw_dashed_line(frame, y, width, color, 1.5);
        
        // Dessiner un label avec le prix limite
        draw_order_label(frame, y, width, order.limit_price, &format!("Limit {}", if order.trade_type == TradeType::Buy { "BUY" } else { "SELL" }), color);
    }
}

/// Dessine les lignes TP/SL des positions ouvertes
pub fn draw_tp_sl_lines(
    frame: &mut Frame,
    viewport: &Viewport,
    positions: &[Position],
    current_symbol: &str,
) {
    for position in positions {
        if position.symbol != current_symbol {
            continue;
        }
        
        // Dessiner Take Profit
        if let Some(tp) = position.take_profit {
            let y = viewport.price_scale().price_to_y(tp);
            
            if y >= -10.0 && y <= viewport.height() + 10.0 {
                let width = viewport.width();
                let color = Color::from_rgba(0.0, 0.7, 0.0, 0.6); // Vert pour TP
                draw_dashed_line(frame, y, width, color, 1.0);
                draw_order_label(frame, y, width, tp, "TP", color);
            }
        }
        
        // Dessiner Stop Loss
        if let Some(sl) = position.stop_loss {
            let y = viewport.price_scale().price_to_y(sl);
            
            if y >= -10.0 && y <= viewport.height() + 10.0 {
                let width = viewport.width();
                let color = Color::from_rgba(0.7, 0.0, 0.0, 0.6); // Rouge pour SL
                draw_dashed_line(frame, y, width, color, 1.0);
                draw_order_label(frame, y, width, sl, "SL", color);
            }
        }
    }
}

/// Dessine une ligne pointillée
fn draw_dashed_line(
    frame: &mut Frame,
    y: f32,
    width: f32,
    color: Color,
    line_width: f32,
) {
    let dash_length = 8.0;
    let gap_length = 4.0;
    
    let mut x = 0.0;
    while x < width {
        let end_x = (x + dash_length).min(width);
        let dash = Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(end_x, y));
        });
        let stroke = Stroke::default()
            .with_color(color)
            .with_width(line_width);
        frame.stroke(&dash, stroke);
        x += dash_length + gap_length;
    }
}

/// Dessine un label pour un ordre/TP/SL
fn draw_order_label(
    frame: &mut Frame,
    y: f32,
    width: f32,
    price: f64,
    label: &str,
    color: Color,
) {
    let badge_width = 70.0;
    let badge_height = 16.0;
    let badge_x = width - badge_width - 5.0;
    let badge_y = y - badge_height / 2.0;
    
    // Fond du badge
    let bg_rect = Path::rectangle(
        Point::new(badge_x, badge_y),
        Size::new(badge_width, badge_height),
    );
    frame.fill(&bg_rect, color);
    
    // Texte du label et prix
    let price_str = format!("{:.2}", price);
    let label_text = format!("{} {}", label, price_str);
    let text = Text {
        content: label_text,
        position: Point::new(badge_x + 4.0, badge_y + 2.0),
        color: Color::WHITE,
        size: iced::Pixels(10.0),
        ..Text::default()
    };
    frame.fill_text(text);
}

/// Dessine les lignes de prévisualisation pour les ordres limit
/// Affiche une ligne au prix limite saisi pour prévisualiser où l'ordre sera placé
pub fn draw_preview_limit_order_lines(
    frame: &mut Frame,
    viewport: &Viewport,
    limit_price: f64,
) {
    let y = viewport.price_scale().price_to_y(limit_price);
    
    // Ne pas dessiner si hors de vue
    if y < -10.0 || y > viewport.height() + 10.0 {
        return;
    }
    
    let width = viewport.width();
    
    // Couleur pour la prévisualisation (jaune/orange pour indiquer que c'est une prévisualisation)
    let preview_color = Color::from_rgba(1.0, 0.8, 0.0, 0.6); // Jaune/orange semi-transparent
    
    // Dessiner une ligne pointillée pour la prévisualisation (style différent des ordres réels)
    // Utiliser des tirets plus longs pour distinguer visuellement
    let dash_length = 12.0;
    let gap_length = 6.0;
    
    let mut x = 0.0;
    while x < width {
        let end_x = (x + dash_length).min(width);
        let dash = Path::new(|builder| {
            builder.move_to(Point::new(x, y));
            builder.line_to(Point::new(end_x, y));
        });
        let stroke = Stroke::default()
            .with_color(preview_color)
            .with_width(1.5);
        frame.stroke(&dash, stroke);
        x += dash_length + gap_length;
    }
    
    // Dessiner un label avec "Preview" pour indiquer que c'est une prévisualisation
    let badge_width = 90.0;
    let badge_height = 18.0;
    let badge_x = width - badge_width - 5.0;
    let badge_y = y - badge_height / 2.0;
    
    // Fond du badge
    let bg_rect = Path::rectangle(
        Point::new(badge_x, badge_y),
        Size::new(badge_width, badge_height),
    );
    frame.fill(&bg_rect, preview_color);
    
    // Texte du label
    let price_str = format!("{:.2}", limit_price);
    let label_text = format!("Preview {}", price_str);
    let text = Text {
        content: label_text,
        position: Point::new(badge_x + 4.0, badge_y + 2.0),
        color: Color::WHITE,
        size: iced::Pixels(10.0),
        ..Text::default()
    };
    frame.fill_text(text);
}

/// Dessine les lignes de prévisualisation pour Take Profit et Stop Loss
/// Affiche les lignes TP (vert) et SL (rouge) au prix saisi pour prévisualiser où ils seront placés
pub fn draw_preview_tp_sl_lines(
    frame: &mut Frame,
    viewport: &Viewport,
    take_profit: Option<f64>,
    stop_loss: Option<f64>,
) {
    let width = viewport.width();
    
    // Dessiner Take Profit si défini
    if let Some(tp) = take_profit {
        let y = viewport.price_scale().price_to_y(tp);
        
        if y >= -10.0 && y <= viewport.height() + 10.0 {
            // Couleur pour la prévisualisation TP (vert plus transparent)
            let tp_color = Color::from_rgba(0.0, 0.7, 0.0, 0.5); // Vert semi-transparent pour prévisualisation
            
            // Dessiner une ligne pointillée pour la prévisualisation
            let dash_length = 12.0;
            let gap_length = 6.0;
            
            let mut x = 0.0;
            while x < width {
                let end_x = (x + dash_length).min(width);
                let dash = Path::new(|builder| {
                    builder.move_to(Point::new(x, y));
                    builder.line_to(Point::new(end_x, y));
                });
                let stroke = Stroke::default()
                    .with_color(tp_color)
                    .with_width(1.5);
                frame.stroke(&dash, stroke);
                x += dash_length + gap_length;
            }
            
            // Dessiner un label avec "Preview TP"
            let badge_width = 85.0;
            let badge_height = 18.0;
            let badge_x = width - badge_width - 5.0;
            let badge_y = y - badge_height / 2.0;
            
            let bg_rect = Path::rectangle(
                Point::new(badge_x, badge_y),
                Size::new(badge_width, badge_height),
            );
            frame.fill(&bg_rect, tp_color);
            
            let price_str = format!("{:.2}", tp);
            let label_text = format!("Preview TP {}", price_str);
            let text = Text {
                content: label_text,
                position: Point::new(badge_x + 4.0, badge_y + 2.0),
                color: Color::WHITE,
                size: iced::Pixels(10.0),
                ..Text::default()
            };
            frame.fill_text(text);
        }
    }
    
    // Dessiner Stop Loss si défini
    if let Some(sl) = stop_loss {
        let y = viewport.price_scale().price_to_y(sl);
        
        if y >= -10.0 && y <= viewport.height() + 10.0 {
            // Couleur pour la prévisualisation SL (rouge plus transparent)
            let sl_color = Color::from_rgba(0.7, 0.0, 0.0, 0.5); // Rouge semi-transparent pour prévisualisation
            
            // Dessiner une ligne pointillée pour la prévisualisation
            let dash_length = 12.0;
            let gap_length = 6.0;
            
            let mut x = 0.0;
            while x < width {
                let end_x = (x + dash_length).min(width);
                let dash = Path::new(|builder| {
                    builder.move_to(Point::new(x, y));
                    builder.line_to(Point::new(end_x, y));
                });
                let stroke = Stroke::default()
                    .with_color(sl_color)
                    .with_width(1.5);
                frame.stroke(&dash, stroke);
                x += dash_length + gap_length;
            }
            
            // Dessiner un label avec "Preview SL"
            let badge_width = 85.0;
            let badge_height = 18.0;
            let badge_x = width - badge_width - 5.0;
            let badge_y = y - badge_height / 2.0;
            
            let bg_rect = Path::rectangle(
                Point::new(badge_x, badge_y),
                Size::new(badge_width, badge_height),
            );
            frame.fill(&bg_rect, sl_color);
            
            let price_str = format!("{:.2}", sl);
            let label_text = format!("Preview SL {}", price_str);
            let text = Text {
                content: label_text,
                position: Point::new(badge_x + 4.0, badge_y + 2.0),
                color: Color::WHITE,
                size: iced::Pixels(10.0),
                ..Text::default()
            };
            frame.fill_text(text);
        }
    }
}

