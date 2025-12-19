use iced::Point;

/// État des interactions utilisateur
#[derive(Debug, Clone, Default)]
pub struct InteractionState {
    /// Position actuelle de la souris
    pub mouse_position: Option<Point>,
    /// Position de départ d'un drag
    pub drag_start: Option<Point>,
    /// Indique si l'utilisateur est en train de faire un pan
    pub is_panning: bool,
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

    /// Termine le pan
    pub fn end_pan(&mut self) {
        self.drag_start = None;
        self.is_panning = false;
    }
}

