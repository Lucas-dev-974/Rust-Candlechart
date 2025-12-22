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
pub mod panel_state;
pub mod resize_handle;
pub mod bottom_panel_sections;
pub mod account_type;

pub use messages::Message;
pub use app_state::ChartApp;

