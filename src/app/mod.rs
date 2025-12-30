//! Module principal de l'application
//! 
//! Ce module contient la structure principale de l'application Iced et sa logique.

// Modules principaux
pub mod window_manager;
pub mod messages;
pub mod app_state;
pub mod views;
pub mod view_styles;

// Modules organisés par domaine
pub mod state;
pub mod persistence;
pub mod data;
pub mod realtime;
pub mod ui;
pub mod handlers;
pub mod utils;
pub mod strategies;

pub use messages::Message;
pub use app_state::ChartApp;

