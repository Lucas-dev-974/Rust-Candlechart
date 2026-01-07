//! Gestion du temps réel
//!
//! Ce module gère les fonctionnalités temps réel (streaming de données, mises à jour en direct).
//!
//! Organisation :
//! - `updates.rs` : Mises à jour en temps réel
//! - `gaps.rs` : Détection et complétion des gaps
//! - `download.rs` : Téléchargement par batch
//! - `save.rs` : Sauvegarde asynchrone
//! - `connection.rs` : Test de connexion
//! - `realtime_utils.rs` : Fonctions utilitaires pures

mod updates;
mod gaps;
mod download;
mod save;
mod connection;
mod realtime_utils;

// Réexporter les fonctions publiques pour compatibilité
pub use updates::{update_realtime, apply_realtime_updates};
pub use gaps::{
    has_gaps_to_fill, auto_complete_series, complete_missing_data,
    apply_complete_missing_data_results, complete_gaps, apply_complete_gaps_results,
};
pub use download::{load_full_history, download_batch};
pub use save::save_series_async;
pub use connection::{test_provider_connection, fetch_account_info, load_assets};



