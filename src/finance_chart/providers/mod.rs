//! Module des providers de données en temps réel
//!
//! Contient les implémentations des différents providers (Binance, etc.)
//! et la gestion de leur configuration.

pub mod binance;
pub mod config;

// Ré-exports
pub use binance::BinanceProvider;
pub use config::{ProviderConfigManager, ProviderType};

