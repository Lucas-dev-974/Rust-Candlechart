use iced::Point;

/// État des interactions utilisateur
#[derive(Debug, Clone, Default)]
pub struct InteractionState {
    /// Position actuelle de la souris
    pub mouse_position: Option<Point>,
    /// Position de départ d'un drag (position relative au graphique principal)
    pub drag_start: Option<Point>,
    /// Indique si l'utilisateur est en train de faire un pan
    pub is_panning: bool,
    /// Bounds du graphique principal (x, y, width, height) pour convertir positions absolues en relatives
    pub main_chart_bounds: Option<(f32, f32, f32, f32)>,
}

impl InteractionState {
    /// Démarre un pan
    pub fn start_pan(&mut self, position: Point) {
        self.drag_start = Some(position);
        self.is_panning = true;
    }

    /// Met à jour le pan
    pub fn update_pan(&mut self, position: Point) -> Option<(f32, f32)> {
        if let Some(start) = self.drag_start {
            let delta_x = position.x - start.x;
            let delta_y = position.y - start.y;
            // Mettre à jour drag_start pour le prochain mouvement
            self.drag_start = Some(position);
            Some((delta_x, delta_y))
        } else {
            None
        }
    }

    /// Met à jour les bounds du graphique principal
    pub fn set_main_chart_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.main_chart_bounds = Some((x, y, width, height));
    }

    /// Convertit une position absolue en position relative au graphique principal
    pub fn absolute_to_relative(&self, absolute_position: Point) -> Point {
        if let Some((x, y, _, _)) = self.main_chart_bounds {
            Point::new(absolute_position.x - x, absolute_position.y - y)
        } else {
            // Si les bounds ne sont pas encore définies, utiliser la position telle quelle
            absolute_position
        }
    }

    /// Termine le pan
    pub fn end_pan(&mut self) {
        self.drag_start = None;
        self.is_panning = false;
    }
}

