//! Vues de l'application Iced
//!
//! Ce module contient toutes les méthodes de rendu (view) pour les différentes fenêtres
//! de l'application : fenêtre principale, settings, et configuration des providers.
//!
//! Structure :
//! - `main.rs` : vue principale avec chart et panneaux
//! - `panels/` : panneaux latéraux (droite et bas)
//!   - `indicators.rs` : panneaux d'indicateurs (Volume, RSI, MACD)
//!   - `right.rs` : panneau de droite
//!   - `bottom.rs` : panneau du bas
//!   - `sections/` : sections du panneau du bas
//! - `indicators.rs` : onglet d'indicateurs
//! - `settings.rs` : fenêtre de configuration du style
//! - `provider.rs` : fenêtre de configuration des providers
//! - `account.rs` : section compte et trading

mod main_view;
mod panels;
mod indicators;
mod settings;
mod provider;
mod account;
mod helpers;
mod downloads_view;
mod context_menu_capture;
mod crosshair_overlay;

// Réexporter les fonctions publiques pour compatibilité
pub use main_view::view_main;
pub use settings::view_settings;
pub use provider::view_provider_config;
pub use downloads_view::view_downloads;

