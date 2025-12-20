use super::super::core::{SeriesManager, SeriesData, Candle, SeriesId};
use super::super::interaction::InteractionState;
use super::super::viewport::Viewport;
use super::super::realtime::{UpdateResult, RealtimeDataProvider};

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

    // ============================================================================
    // Mises à jour en temps réel
    // ============================================================================

    /// Met à jour ou ajoute une bougie à une série spécifique
    ///
    /// Si la bougie a le même timestamp que la dernière bougie de la série,
    /// elle sera mise à jour. Sinon, une nouvelle bougie sera ajoutée.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série à mettre à jour
    /// * `candle` - Nouvelle bougie à ajouter ou mettre à jour
    ///
    /// # Retourne
    /// Le résultat de la mise à jour
    pub fn update_candle(&mut self, series_id: &SeriesId, candle: Candle) -> UpdateResult {
        match self.series_manager.update_series_candle(series_id, candle) {
            Some(Ok(true)) => UpdateResult::CandleUpdated,
            Some(Ok(false)) => UpdateResult::NewCandle,
            Some(Err(e)) => UpdateResult::Error(format!("Bougie invalide: {}", e)),
            None => UpdateResult::Error(format!("Série {} introuvable", series_id.name)),
        }
    }

    /// Fusionne plusieurs bougies dans une série spécifique
    ///
    /// Les bougies sont fusionnées intelligemment : celles avec le même timestamp
    /// remplacent les existantes, les nouvelles sont insérées dans l'ordre chronologique.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série à mettre à jour
    /// * `candles` - Liste des bougies à fusionner
    ///
    /// # Retourne
    /// Le résultat de la fusion avec le nombre de nouvelles bougies ajoutées
    pub fn merge_candles(&mut self, series_id: &SeriesId, candles: Vec<Candle>) -> UpdateResult {
        match self.series_manager.merge_series_candles(series_id, candles) {
            Some(added) => UpdateResult::MultipleCandlesAdded(added),
            None => UpdateResult::Error(format!("Série {} introuvable", series_id.name)),
        }
    }

    /// Met à jour les données depuis un fournisseur de données en temps réel
    ///
    /// Cette méthode récupère la dernière bougie depuis le provider et la met à jour
    /// dans la série correspondante.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série à mettre à jour
    /// * `provider` - Fournisseur de données en temps réel
    ///
    /// # Retourne
    /// Le résultat de la mise à jour
    pub fn update_from_provider<P: RealtimeDataProvider>(
        &mut self,
        series_id: &SeriesId,
        provider: &P,
    ) -> UpdateResult {
        match provider.fetch_latest_candle(series_id) {
            Ok(Some(candle)) => {
                let result = self.update_candle(series_id, candle);
                // Mettre à jour le viewport si nécessaire (si on est sur la dernière bougie)
                if matches!(result, UpdateResult::NewCandle | UpdateResult::CandleUpdated) {
                    // Optionnel : ajuster le viewport pour suivre les nouvelles données
                    // self.auto_scroll_to_latest();
                }
                result
            }
            Ok(None) => UpdateResult::NoUpdate,
            Err(e) => UpdateResult::Error(e),
        }
    }

    /// Synchronise une série complète depuis un fournisseur de données
    ///
    /// Récupère toutes les bougies depuis le provider et les fusionne avec les données existantes.
    /// Utile pour la première connexion ou une resynchronisation complète.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série à synchroniser
    /// * `provider` - Fournisseur de données en temps réel
    ///
    /// # Retourne
    /// Le résultat de la synchronisation
    pub fn sync_from_provider<P: RealtimeDataProvider>(
        &mut self,
        series_id: &SeriesId,
        provider: &P,
    ) -> UpdateResult {
        match provider.fetch_all_candles(series_id) {
            Ok(candles) => {
                if candles.is_empty() {
                    UpdateResult::NoUpdate
                } else {
                    self.merge_candles(series_id, candles)
                }
            }
            Err(e) => UpdateResult::Error(e),
        }
    }

    /// Récupère les nouvelles bougies depuis un timestamp donné
    ///
    /// Utile pour récupérer plusieurs bougies manquantes d'un coup.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série
    /// * `since_timestamp` - Timestamp à partir duquel récupérer les nouvelles bougies
    /// * `provider` - Fournisseur de données en temps réel
    ///
    /// # Retourne
    /// Le résultat de la récupération
    pub fn fetch_new_candles_from_provider<P: RealtimeDataProvider>(
        &mut self,
        series_id: &SeriesId,
        since_timestamp: i64,
        provider: &P,
    ) -> UpdateResult {
        match provider.fetch_new_candles(series_id, since_timestamp) {
            Ok(candles) => {
                if candles.is_empty() {
                    UpdateResult::NoUpdate
                } else {
                    self.merge_candles(series_id, candles)
                }
            }
            Err(e) => UpdateResult::Error(e),
        }
    }

    /// Ajuste automatiquement le viewport pour afficher les dernières données
    ///
    /// Utile après une mise à jour en temps réel pour suivre les nouvelles bougies.
    pub fn auto_scroll_to_latest(&mut self) {
        if let Some(active_series) = self.series_manager.active_series().next() {
            // Si on est déjà proche de la fin, ajuster pour montrer les nouvelles données
            if let Some(max_time) = active_series.data.max_timestamp() {
                let (current_min, current_max) = self.viewport.time_scale().time_range();
                // Si on est dans les 10% de la fin, ajuster pour suivre
                let range = current_max - current_min;
                if max_time > current_max - (range / 10) {
                    // Ajuster le viewport pour montrer les dernières données
                    self.viewport.focus_on_recent(&active_series.data, DEFAULT_VISIBLE_CANDLES);
                }
            }
        }
    }
}
