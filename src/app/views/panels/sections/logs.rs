//! Section "Logs"

use iced::Element;
use crate::app::{app_state::ChartApp, messages::Message};
use super::super::super::helpers::simple_panel_section;

/// Vue pour la section "Logs"
pub fn view_logs(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Logs",
        "Cette section affiche les logs de l'application."
    )
}



