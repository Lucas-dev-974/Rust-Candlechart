//! Handlers pour la gestion des indicateurs et panneaux

use iced::Task;
use crate::app::app_state::ChartApp;

/// Gère le toggle des panneaux d'indicateurs
pub fn handle_toggle_volume_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.volume.toggle_visibility();
    app.save_panel_state();
    Task::none()
}

pub fn handle_toggle_rsi_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.rsi.toggle_visibility();
    app.save_panel_state();
    Task::none()
}

pub fn handle_toggle_macd_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.macd.toggle_visibility();
    app.save_panel_state();
    Task::none()
}

/// Gère le toggle des indicateurs
pub fn handle_toggle_bollinger_bands(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.indicators.bollinger_bands_enabled = !app.indicators.bollinger_bands_enabled;
    Task::none()
}

pub fn handle_toggle_moving_average(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.indicators.moving_average_enabled = !app.indicators.moving_average_enabled;
    Task::none()
}

/// Gère la mise à jour des paramètres des indicateurs
pub fn handle_update_rsi_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.rsi_period = period;
    Task::none()
}

pub fn handle_update_rsi_method(app: &mut ChartApp, method: crate::app::state::RSIMethod) -> Task<crate::app::messages::Message> {
    app.indicators.params.rsi_method = method;
    Task::none()
}

pub fn handle_update_macd_fast_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.macd_fast_period = period;
    Task::none()
}

pub fn handle_update_macd_slow_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.macd_slow_period = period;
    Task::none()
}

pub fn handle_update_macd_signal_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.macd_signal_period = period;
    Task::none()
}

pub fn handle_update_bollinger_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.bollinger_period = period;
    Task::none()
}

pub fn handle_update_bollinger_std_dev(app: &mut ChartApp, std_dev: f64) -> Task<crate::app::messages::Message> {
    app.indicators.params.bollinger_std_dev = std_dev;
    Task::none()
}

pub fn handle_update_ma_period(app: &mut ChartApp, period: usize) -> Task<crate::app::messages::Message> {
    app.indicators.params.ma_period = period;
    Task::none()
}

/// Gère le redimensionnement des panneaux
pub fn handle_start_resize_right_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.right.start_resize(pos);
    Task::none()
}

pub fn handle_start_resize_bottom_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.bottom.start_resize(pos);
    Task::none()
}

pub fn handle_update_resize_right_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.right.update_resize(pos, true);
    Task::none()
}

pub fn handle_update_resize_bottom_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.bottom.update_resize(pos, false);
    Task::none()
}

pub fn handle_end_resize_right_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.right.end_resize();
    app.save_panel_state();
    Task::none()
}

pub fn handle_end_resize_bottom_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.bottom.end_resize();
    app.save_panel_state();
    Task::none()
}

pub fn handle_start_resize_volume_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.volume.start_resize(pos);
    Task::none()
}

pub fn handle_update_resize_volume_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.volume.update_resize(pos, false);
    Task::none()
}

pub fn handle_end_resize_volume_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.volume.end_resize();
    app.save_panel_state();
    Task::none()
}

pub fn handle_start_resize_rsi_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.rsi.start_resize(pos);
    Task::none()
}

pub fn handle_start_resize_macd_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.macd.start_resize(pos);
    Task::none()
}

pub fn handle_update_resize_rsi_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.rsi.update_resize(pos, false);
    Task::none()
}

pub fn handle_update_resize_macd_panel(app: &mut ChartApp, pos: f32) -> Task<crate::app::messages::Message> {
    app.ui.panels.macd.update_resize(pos, false);
    Task::none()
}

pub fn handle_end_resize_rsi_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.rsi.end_resize();
    app.save_panel_state();
    Task::none()
}

pub fn handle_end_resize_macd_panel(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.macd.end_resize();
    app.save_panel_state();
    Task::none()
}

/// Gère la sélection et le déplacement des sections
pub fn handle_select_bottom_section(
    app: &mut ChartApp,
    section: crate::app::state::BottomPanelSection
) -> Task<crate::app::messages::Message> {
    app.ui.bottom_panel_sections.set_active_section(section);
    app.ui.section_context_menu = None;
    Task::none()
}

pub fn handle_select_right_section(
    app: &mut ChartApp,
    section: crate::app::state::BottomPanelSection
) -> Task<crate::app::messages::Message> {
    app.ui.bottom_panel_sections.set_active_right_section(section);
    app.ui.section_context_menu = None;
    Task::none()
}

pub fn handle_open_section_context_menu(
    app: &mut ChartApp,
    section: crate::app::state::BottomPanelSection,
    position: iced::Point
) -> Task<crate::app::messages::Message> {
    app.ui.section_context_menu = Some((section, position));
    Task::none()
}

pub fn handle_close_section_context_menu(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.section_context_menu = None;
    Task::none()
}

pub fn handle_move_section_to_right_panel(
    app: &mut ChartApp,
    section: crate::app::state::BottomPanelSection
) -> Task<crate::app::messages::Message> {
    app.ui.bottom_panel_sections.move_to_right_panel(section);
    app.ui.section_context_menu = None;
    app.save_panel_state();
    Task::none()
}

pub fn handle_move_section_to_bottom_panel(
    app: &mut ChartApp,
    section: crate::app::state::BottomPanelSection
) -> Task<crate::app::messages::Message> {
    app.ui.bottom_panel_sections.move_to_bottom_panel(section);
    app.ui.section_context_menu = None;
    app.save_panel_state();
    Task::none()
}

/// Gère le focus des panneaux
pub fn handle_set_right_panel_focus(app: &mut ChartApp, focused: bool) -> Task<crate::app::messages::Message> {
    app.ui.panels.right.set_focused(focused);
    Task::none()
}

pub fn handle_set_bottom_panel_focus(app: &mut ChartApp, focused: bool) -> Task<crate::app::messages::Message> {
    app.ui.panels.bottom.set_focused(focused);
    Task::none()
}

pub fn handle_set_volume_panel_focus(app: &mut ChartApp, focused: bool) -> Task<crate::app::messages::Message> {
    app.ui.panels.volume.set_focused(focused);
    Task::none()
}

pub fn handle_set_rsi_panel_focus(app: &mut ChartApp, focused: bool) -> Task<crate::app::messages::Message> {
    app.ui.panels.rsi.set_focused(focused);
    Task::none()
}

pub fn handle_set_macd_panel_focus(app: &mut ChartApp, focused: bool) -> Task<crate::app::messages::Message> {
    app.ui.panels.macd.set_focused(focused);
    Task::none()
}

pub fn handle_clear_panel_focus(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.ui.panels.right.set_focused(false);
    app.ui.panels.bottom.set_focused(false);
    app.ui.panels.volume.set_focused(false);
    app.ui.panels.rsi.set_focused(false);
    Task::none()
}

