//! Gestion des données
//!
//! Ce module gère le chargement, le téléchargement et l'historique des données.

pub mod data_loading;
mod download_manager;
mod trade_history;

pub use download_manager::DownloadManager;
pub use trade_history::{TradeHistory, Trade, TradeType, Position, OrderType, PendingOrder};



