//! Rendu des rectangles dessinés sur le graphique

use iced::widget::canvas::{self, Frame, Path};
use iced::{Color, Point, Size};

use crate::finance_chart::tools::{DrawnRectangle, HANDLE_SIZE};
use crate::finance_chart::viewport::Viewport;

/// Dessine un rectangle avec sa bordure et ses poignées si sélectionné
pub fn draw_rectangle(
    frame: &mut Frame,
    viewport: &Viewport,
    rect: &DrawnRectangle,
    is_selected: bool,
) {
    let time_scale = viewport.time_scale();
    let price_scale = viewport.price_scale();

    // Convertir les coordonnées graphique en coordonnées écran
    let x1 = time_scale.time_to_x(rect.start_time);
    let x2 = time_scale.time_to_x(rect.end_time);
    let y1 = price_scale.price_to_y(rect.start_price);
    let y2 = price_scale.price_to_y(rect.end_price);

    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);
    let width = max_x - min_x;
    let height = max_y - min_y;

    // Ne pas dessiner si le rectangle est hors de vue
    if max_x < 0.0 || min_x > viewport.width() ||
       max_y < 0.0 || min_y > viewport.height() {
        return;
    }

    // Dessiner le fond
    let rect_path = Path::rectangle(
        Point::new(min_x, min_y),
        Size::new(width, height),
    );
    frame.fill(&rect_path, rect.color);

    // Dessiner la bordure (plus opaque si sélectionné)
    let border_color = if is_selected {
        Color::from_rgba(1.0, 1.0, 1.0, 1.0) // Blanc si sélectionné
    } else {
        Color::from_rgba(
            rect.color.r,
            rect.color.g,
            rect.color.b,
            (rect.color.a * 2.5).min(1.0),
        )
    };
    let stroke = canvas::Stroke::default()
        .with_color(border_color)
        .with_width(if is_selected { 2.0 } else { 1.5 });
    frame.stroke(&rect_path, stroke);

    // Dessiner les poignées de redimensionnement si sélectionné
    if is_selected {
        draw_handles(frame, min_x, min_y, max_x, max_y);
    }
}

/// Dessine l'aperçu d'un rectangle en cours de dessin
pub fn draw_preview_rectangle(
    frame: &mut Frame,
    start_x: f32,
    start_y: f32,
    current_x: f32,
    current_y: f32,
) {
    let min_x = start_x.min(current_x);
    let min_y = start_y.min(current_y);
    let width = (current_x - start_x).abs();
    let height = (current_y - start_y).abs();

    if width > 1.0 && height > 1.0 {
        // Fond semi-transparent
        let preview_rect = Path::rectangle(
            Point::new(min_x, min_y),
            Size::new(width, height),
        );
        frame.fill(&preview_rect, Color::from_rgba(0.2, 0.6, 1.0, 0.2));

        // Bordure
        let stroke = canvas::Stroke::default()
            .with_color(Color::from_rgba(0.2, 0.6, 1.0, 0.8))
            .with_width(1.5);
        frame.stroke(&preview_rect, stroke);
    }
}

/// Dessine les poignées de redimensionnement aux coins et sur les bords
fn draw_handles(frame: &mut Frame, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
    let handle_color = Color::WHITE;
    let handle_border = Color::from_rgba(0.2, 0.6, 1.0, 1.0);
    let half = HANDLE_SIZE / 2.0;
    let mid_x = (min_x + max_x) / 2.0;
    let mid_y = (min_y + max_y) / 2.0;

    // Positions des 8 poignées
    let handles = [
        (min_x, min_y),         // TopLeft
        (max_x, min_y),         // TopRight
        (min_x, max_y),         // BottomLeft
        (max_x, max_y),         // BottomRight
        (mid_x, min_y),         // Top
        (mid_x, max_y),         // Bottom
        (min_x, mid_y),         // Left
        (max_x, mid_y),         // Right
    ];

    for (hx, hy) in handles {
        let handle_rect = Path::rectangle(
            Point::new(hx - half, hy - half),
            Size::new(HANDLE_SIZE, HANDLE_SIZE),
        );
        frame.fill(&handle_rect, handle_color);
        let stroke = canvas::Stroke::default()
            .with_color(handle_border)
            .with_width(1.0);
        frame.stroke(&handle_rect, stroke);
    }
}

