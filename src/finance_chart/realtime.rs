//! Module pour la mise à jour en temps réel des données du graphique
//!
//! Fournit des abstractions et des fonctions pour intégrer des sources de données
//! externes (API, WebSocket, etc.) avec le graphique.
//!
//! # Exemple d'utilisation
//!
//! ```ignore
//! use candlechart::{
//!     ChartState, RealtimeDataProvider,
//!     core::{SeriesId, Candle},
//! };
//!
//! // 1. Implémenter le trait RealtimeDataProvider pour votre API
//! struct MyApiProvider {
//!     api_client: MyApiClient,
//! }
//!
//! impl RealtimeDataProvider for MyApiProvider {
//!     fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
//!         // Appel à votre API
//!         self.api_client.get_latest_candle(series_id)
//!     }
//!
//!     fn fetch_new_candles(&self, series_id: &SeriesId, since: i64) -> Result<Vec<Candle>, String> {
//!         // Appel à votre API
//!         self.api_client.get_candles_since(series_id, since)
//!     }
//! }
//!
//! // 2. Utiliser dans votre application
//! let mut chart_state = ChartState::new(1200.0, 800.0);
//! let provider = MyApiProvider { api_client: MyApiClient::new() };
//! let series_id = SeriesId::new("BTCUSDT_1h");
//!
//! // Mettre à jour avec la dernière bougie
//! let result = chart_state.update_from_provider(&series_id, &provider);
//! match result {
//!     UpdateResult::NewCandle => println!("Nouvelle bougie ajoutée"),
//!     UpdateResult::CandleUpdated => println!("Bougie mise à jour"),
//!     UpdateResult::NoUpdate => println!("Aucune mise à jour"),
//!     UpdateResult::Error(e) => eprintln!("Erreur: {}", e),
//!     _ => {}
//! }
//!
//! // Synchroniser toutes les bougies
//! chart_state.sync_from_provider(&series_id, &provider);
//!
//! // Récupérer les nouvelles bougies depuis un timestamp
//! if let Some(series) = chart_state.series_manager.get_series(&series_id) {
//!     let last_ts = series.data.max_timestamp().unwrap_or(0);
//!     chart_state.fetch_new_candles_from_provider(&series_id, last_ts, &provider);
//! }
//! ```

pub mod error;

use super::core::{Candle, SeriesId};
pub use error::ProviderError;

/// Résultat d'une mise à jour en temps réel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateResult {
    /// Aucune mise à jour nécessaire
    #[allow(dead_code)] // Utilisé dans les méthodes update_from_provider, sync_from_provider, fetch_new_candles_from_provider
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
///
/// # Exemple d'implémentation
///
/// ```ignore
/// struct ApiDataProvider {
///     api_client: ApiClient,
/// }
///
/// impl RealtimeDataProvider for ApiDataProvider {
///     fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
///         // Appel API pour récupérer la dernière bougie
///         Ok(Some(self.api_client.get_latest_candle(series_id)?))
///     }
///
///     fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String> {
///         // Appel API pour récupérer les nouvelles bougies depuis un timestamp
///         Ok(self.api_client.get_candles_since(series_id, since_timestamp)?)
///     }
/// } 
/// ```
#[allow(dead_code)] // Trait public pour implémentation par les providers
pub trait RealtimeDataProvider {
    /// Récupère la dernière bougie pour une série donnée
    ///
    /// Retourne `Ok(None)` si aucune bougie n'est disponible,
    /// `Ok(Some(candle))` si une bougie est disponible,
    /// `Err(msg)` en cas d'erreur.
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String>;

    /// Récupère les nouvelles bougies pour une série depuis un timestamp donné
    ///
    /// Retourne toutes les bougies avec un timestamp >= `since_timestamp`.
    /// Utile pour récupérer plusieurs bougies manquantes d'un coup.
    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String>;

    /// Récupère toutes les bougies pour une série (pour synchronisation complète)
    ///
    /// Utile pour la première connexion ou pour une resynchronisation complète.
    fn fetch_all_candles(&self, series_id: &SeriesId) -> Result<Vec<Candle>, String> {
        // Par défaut, récupère depuis le timestamp 0
        self.fetch_new_candles(series_id, 0)
    }
}


