//! Section "Compte"

use iced::Element;
use crate::app::{app_state::ChartApp, messages::Message};
use super::super::super::account::view_bottom_panel_account;

/// Vue pour la section "Compte"
pub fn view_account(app: &ChartApp) -> Element<'_, Message> {
    view_bottom_panel_account(app)
}



