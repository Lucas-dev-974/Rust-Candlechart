//! Module principal de l'application
//! 
//! Ce module contient la structure principale de l'application Iced et sa logique.

pub mod constants;
pub mod utils;
pub mod window_manager;
pub mod messages;
pub mod app_state;
pub mod data_loading;
pub mod realtime;
pub mod handlers;
pub mod views;

pub use messages::Message;
pub use app_state::ChartApp;

