//! Handlers pour la gestion des panneaux et outils

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::finance_chart::{YAxisMessage, XAxisMessage, ToolsPanelMessage};

/// Gère les messages des axes
pub fn handle_yaxis_message(app: &mut ChartApp, msg: YAxisMessage) -> Task<crate::app::messages::Message> {
    match msg {
        YAxisMessage::ZoomVertical { factor } => {
            app.chart_state.zoom_vertical(factor);
        }
    }
    Task::none()
}

pub fn handle_xaxis_message(app: &mut ChartApp, msg: XAxisMessage) -> Task<crate::app::messages::Message> {
    match msg {
        XAxisMessage::ZoomHorizontal { factor } => {
            app.chart_state.zoom(factor);
        }
    }
    Task::none()
}

/// Gère les messages du panel d'outils
pub fn handle_tools_panel_message(app: &mut ChartApp, msg: ToolsPanelMessage) -> Task<crate::app::messages::Message> {
    match msg {
        ToolsPanelMessage::ToggleTool { tool } => {
            if app.tools_state.selected_tool == Some(tool) {
                app.tools_state.selected_tool = None;
            } else {
                app.tools_state.selected_tool = Some(tool);
            }
        }
        ToolsPanelMessage::ToggleIndicatorsPanel => {
            app.ui.indicators_panel_open = !app.ui.indicators_panel_open;
        }
    }
    Task::none()
}

