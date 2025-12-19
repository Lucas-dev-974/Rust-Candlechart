use super::super::core::{SeriesManager, SeriesData};
use super::super::interaction::InteractionState;
use super::super::viewport::Viewport;

/// Nombre de bougies visibles par défaut à l'initialisation
const DEFAULT_VISIBLE_CANDLES: usize = 150;

/// État complet du graphique
/// 
/// Gère les données, le viewport et les interactions.
#[derive(Debug, Clone)]
pub struct ChartState {
    /// Gestionnaire de plusieurs séries temporelles
    pub series_manager: SeriesManager,
    /// Viewport actuel
    pub viewport: Viewport,
    /// État des interactions
    pub interaction: InteractionState,
}

impl ChartState {
    /// Crée un nouvel état de graphique
    pub fn new(width: f32, height: f32) -> Self {
        let viewport = Viewport::new(width, height);
        
        Self {
            series_manager: SeriesManager::new(),
            viewport,
            interaction: InteractionState::default(),
        }
    }

    /// Ajoute une série au graphique
    pub fn add_series(&mut self, series: SeriesData) {
        self.series_manager.add_series(series);
        // Mettre à jour le viewport avec la plage globale après ajout
        self.update_viewport_from_series();
    }

    /// Met à jour le viewport en fonction des séries actives
    /// Réinitialise le zoom pour afficher correctement la série active
    pub fn update_viewport_from_series(&mut self) {
        // Puisqu'on n'affiche qu'une seule série à la fois, utiliser focus_on_recent
        // qui réinitialise correctement le zoom horizontal et vertical
        if let Some(active_series) = self.series_manager.active_series().next() {
            // Utiliser focus_on_recent qui calcule automatiquement :
            // - La plage temporelle pour les N dernières bougies
            // - La plage de prix pour les bougies visibles
            // Cela réinitialise complètement le zoom
            self.viewport.focus_on_recent(&active_series.data, DEFAULT_VISIBLE_CANDLES);
        }
    }

    /// Met à jour la taille du viewport
    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport.set_size(width, height);
    }

    /// Retourne les bougies visibles dans le viewport actuel pour toutes les séries actives
    pub fn visible_candles(&self) -> Vec<(super::super::core::SeriesId, &[super::super::core::Candle])> {
        let (min_time, max_time) = self.viewport.time_scale().time_range();
        self.series_manager.visible_candles(min_time..max_time)
    }

    /// Retourne la dernière bougie de la première série active (pour la ligne de prix courant)
    pub fn last_candle(&self) -> Option<&super::super::core::Candle> {
        self.series_manager.active_series()
            .next()
            .and_then(|series| series.data.last_candle())
    }

    /// Effectue un pan horizontal (déplacement temporel)
    pub fn pan_horizontal(&mut self, delta_x: f32) {
        self.viewport.pan_horizontal(delta_x);
    }

    /// Effectue un pan vertical (déplacement prix)
    pub fn pan_vertical(&mut self, delta_y: f32) {
        self.viewport.pan_vertical(delta_y);
    }

    /// Effectue un zoom horizontal (axe X / temps)
    pub fn zoom(&mut self, factor: f64) {
        self.viewport.zoom(factor);
    }

    /// Effectue un zoom vertical (axe Y / prix) - ALT + molette
    pub fn zoom_vertical(&mut self, factor: f64) {
        self.viewport.zoom_vertical(factor);
    }

    /// Effectue un zoom sur les deux axes - CTRL + molette
    pub fn zoom_both(&mut self, factor: f64) {
        self.viewport.zoom_both(factor);
    }

    /// Démarre un pan (drag)
    pub fn start_pan(&mut self, position: iced::Point) {
        self.interaction.start_pan(position);
    }

    /// Met à jour le pan en cours
    pub fn update_pan(&mut self, position: iced::Point) {
        if let Some((delta_x, delta_y)) = self.interaction.update_pan(position) {
            // Inverser le delta horizontal pour un comportement naturel
            // (quand on tire vers la droite,    on voit les données précédentes)
            // Le delta vertical n'est pas inversé (tirer vers le haut = monter)
            self.pan_horizontal(-delta_x);
            self.pan_vertical(delta_y);
        }
    }

    /// Termine le pan
    pub fn end_pan(&mut self) {
        self.interaction.end_pan();
    }
}
