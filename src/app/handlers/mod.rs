//! Gestionnaires d'événements
//!
//! Ce module contient les gestionnaires d'événements de l'application.

mod handlers;
mod windows;
mod series;
mod indicators;
mod settings;
mod provider;
mod trading;
mod realtime;
mod panels;

pub use handlers::handle_chart_message;
pub use windows::{
    handle_open_settings,
    handle_open_downloads,
    handle_window_closed,
};
pub use series::{
    handle_select_series_by_name,
    handle_load_series_complete,
};
pub use indicators::*;
pub use settings::*;
pub use provider::*;
pub use trading::*;
pub use realtime::*;
pub use panels::*;



