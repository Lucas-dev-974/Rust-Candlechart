//! Module pour la mise à jour en temps réel des données du graphique
//!
//! Fournit des abstractions et des fonctions pour intégrer des sources de données
//! externes (API, WebSocket, etc.) avec le graphique.

pub mod error;

use crate::finance_chart::core::{Candle, SeriesId};
pub use error::ProviderError;

/// Résultat d'une mise à jour en temps réel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateResult {
    /// Aucune mise à jour nécessaire
    #[allow(dead_code)]
    NoUpdate,
    /// Nouvelle bougie ajoutée
    NewCandle,
    /// Bougie existante mise à jour
    CandleUpdated,
    /// Plusieurs bougies ajoutées
    MultipleCandlesAdded(usize),
    /// Erreur lors de la mise à jour
    Error(String),
}

/// Trait pour les fournisseurs de données en temps réel
///
/// Ce trait permet d'abstraire la source de données (API REST, WebSocket, etc.)
/// et de l'intégrer avec le système de mise à jour du graphique.
#[allow(dead_code)]
pub trait RealtimeDataProvider {
    /// Récupère la dernière bougie pour une série donnée
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String>;

    /// Récupère les nouvelles bougies pour une série depuis un timestamp donné
    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String>;

    /// Récupère toutes les bougies pour une série (pour synchronisation complète)
    fn fetch_all_candles(&self, series_id: &SeriesId) -> Result<Vec<Candle>, String> {
        self.fetch_new_candles(series_id, 0)
    }
}

