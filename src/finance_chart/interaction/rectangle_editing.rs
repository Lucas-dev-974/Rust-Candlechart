//! Logique d'édition et de hit-testing des rectangles

use iced::{mouse, Point};

use crate::finance_chart::tools::{DrawnRectangle, EditMode, EditState, HANDLE_SIZE};
use crate::finance_chart::viewport::Viewport;

/// Épaisseur de la zone de détection des bords (en pixels)
const BORDER_THICKNESS: f32 = 6.0;

/// Résultat d'un hit-test sur les rectangles
#[derive(Debug, Clone, Copy)]
pub struct HitTestResult {
    pub index: usize,
    pub mode: EditMode,
}

/// Bounds d'un rectangle pour le hit-testing
struct RectBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    mid_x: f32,
    mid_y: f32,
    half_handle: f32,
}

/// Détecte si un point est sur un rectangle et retourne l'index et le mode d'édition
pub fn hit_test_rectangles(
    pos: Point,
    rectangles: &[DrawnRectangle],
    selected_index: Option<usize>,
    viewport: &Viewport,
) -> Option<HitTestResult> {
    let time_scale = viewport.time_scale();
    let price_scale = viewport.price_scale();

    // Parcourir les rectangles en ordre inverse (le dernier dessiné est au-dessus)
    for (index, rect) in rectangles.iter().enumerate().rev() {
        let x1 = time_scale.time_to_x(rect.start_time);
        let x2 = time_scale.time_to_x(rect.end_time);
        let y1 = price_scale.price_to_y(rect.start_price);
        let y2 = price_scale.price_to_y(rect.end_price);

        let bounds = RectBounds {
            min_x: x1.min(x2),
            max_x: x1.max(x2),
            min_y: y1.min(y2),
            max_y: y1.max(y2),
            mid_x: (x1.min(x2) + x1.max(x2)) / 2.0,
            mid_y: (y1.min(y2) + y1.max(y2)) / 2.0,
            half_handle: HANDLE_SIZE / 2.0,
        };

        let is_selected = selected_index == Some(index);

        // Vérifier les poignées (priorité aux coins) - seulement si sélectionné
        if is_selected {
            if let Some(mode) = check_handles(pos, &bounds) {
                return Some(HitTestResult { index, mode });
            }
        }

        // Vérifier si on est à l'intérieur du rectangle
        let inside_x = pos.x >= bounds.min_x && pos.x <= bounds.max_x;
        let inside_y = pos.y >= bounds.min_y && pos.y <= bounds.max_y;
        
        if inside_x && inside_y {
            if is_selected {
                // Si le rectangle est sélectionné, toute la zone intérieure est active
                return Some(HitTestResult { index, mode: EditMode::Move });
            } else {
                // Si non sélectionné, seuls les bords permettent la sélection
                let near_left = pos.x <= bounds.min_x + BORDER_THICKNESS;
                let near_right = pos.x >= bounds.max_x - BORDER_THICKNESS;
                let near_top = pos.y <= bounds.min_y + BORDER_THICKNESS;
                let near_bottom = pos.y >= bounds.max_y - BORDER_THICKNESS;
                
                if near_left || near_right || near_top || near_bottom {
                    return Some(HitTestResult { index, mode: EditMode::Move });
                }
            }
        }
    }

    None
}

/// Vérifie si le point est sur une des 8 poignées
fn check_handles(pos: Point, b: &RectBounds) -> Option<EditMode> {
    let handles: [(f32, f32, EditMode); 8] = [
        (b.min_x, b.min_y, EditMode::ResizeTopLeft),
        (b.max_x, b.min_y, EditMode::ResizeTopRight),
        (b.min_x, b.max_y, EditMode::ResizeBottomLeft),
        (b.max_x, b.max_y, EditMode::ResizeBottomRight),
        (b.mid_x, b.min_y, EditMode::ResizeTop),
        (b.mid_x, b.max_y, EditMode::ResizeBottom),
        (b.min_x, b.mid_y, EditMode::ResizeLeft),
        (b.max_x, b.mid_y, EditMode::ResizeRight),
    ];

    let margin = b.half_handle + 2.0;
    for (hx, hy, mode) in handles {
        if pos.x >= hx - margin && pos.x <= hx + margin &&
           pos.y >= hy - margin && pos.y <= hy + margin {
            return Some(mode);
        }
    }

    None
}

/// Applique une mise à jour d'édition sur un rectangle
pub fn apply_edit_update(
    rect: &mut DrawnRectangle,
    edit_state: &EditState,
    current_time: i64,
    current_price: f64,
) {
    let Some(mode) = edit_state.edit_mode else { return };
    let Some(start_time) = edit_state.start_time else { return };
    let Some(start_price) = edit_state.start_price else { return };
    let Some(ref original) = edit_state.original_rect else { return };

    let delta_time = current_time - start_time;
    let delta_price = current_price - start_price;

    match mode {
        EditMode::Move => {
            rect.start_time = original.start_time + delta_time;
            rect.end_time = original.end_time + delta_time;
            rect.start_price = original.start_price + delta_price;
            rect.end_price = original.end_price + delta_price;
        }
        EditMode::ResizeTopLeft => {
            rect.start_time = original.start_time + delta_time;
            rect.start_price = original.start_price + delta_price;
        }
        EditMode::ResizeTopRight => {
            rect.end_time = original.end_time + delta_time;
            rect.start_price = original.start_price + delta_price;
        }
        EditMode::ResizeBottomLeft => {
            rect.start_time = original.start_time + delta_time;
            rect.end_price = original.end_price + delta_price;
        }
        EditMode::ResizeBottomRight => {
            rect.end_time = original.end_time + delta_time;
            rect.end_price = original.end_price + delta_price;
        }
        EditMode::ResizeTop => {
            rect.start_price = original.start_price + delta_price;
        }
        EditMode::ResizeBottom => {
            rect.end_price = original.end_price + delta_price;
        }
        EditMode::ResizeLeft => {
            rect.start_time = original.start_time + delta_time;
        }
        EditMode::ResizeRight => {
            rect.end_time = original.end_time + delta_time;
        }
    }
}

/// Retourne le curseur approprié pour un mode d'édition
pub fn cursor_for_edit_mode(mode: EditMode) -> mouse::Interaction {
    match mode {
        EditMode::Move => mouse::Interaction::Grabbing,
        EditMode::ResizeTopLeft | EditMode::ResizeBottomRight => mouse::Interaction::Crosshair,
        EditMode::ResizeTopRight | EditMode::ResizeBottomLeft => mouse::Interaction::Crosshair,
        EditMode::ResizeTop | EditMode::ResizeBottom => mouse::Interaction::ResizingVertically,
        EditMode::ResizeLeft | EditMode::ResizeRight => mouse::Interaction::ResizingHorizontally,
    }
}

