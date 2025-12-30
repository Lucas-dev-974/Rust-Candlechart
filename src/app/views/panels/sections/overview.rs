//! Section "Vue d'ensemble"

use iced::Element;
use crate::app::{app_state::ChartApp, messages::Message};
use super::super::super::helpers::simple_panel_section;

/// Vue pour la section "Vue d'ensemble"
pub fn view_overview(_app: &ChartApp) -> Element<'_, Message> {
    simple_panel_section(
        "Vue d'ensemble",
        "Cette section affiche une vue d'ensemble des statistiques et informations principales."
    )
}



