//! Handlers pour la gestion des settings et configuration

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::app::window_manager::WindowType;
use crate::finance_chart::settings::color_fields;

/// Gère la sélection d'une couleur dans les settings
pub fn handle_select_color(
    app: &mut ChartApp,
    field_index: usize,
    color: crate::finance_chart::settings::SerializableColor
) -> Task<crate::app::messages::Message> {
    if let Some(ref mut style) = app.editing_style {
        let fields = color_fields();
        if field_index < fields.len() {
            (fields[field_index].set)(style, color);
        }
    }
    app.editing_color_index = None;
    Task::none()
}

/// Gère l'application des settings
pub fn handle_apply_settings(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use iced::window;
    
    if let Some(new_style) = app.editing_style.take() {
        app.chart_style = new_style.clone();
        if let Err(e) = new_style.save_to_file("chart_style.json") {
            eprintln!("⚠️ Erreur sauvegarde style: {}", e);
        } else {
            println!("✅ Style sauvegardé dans chart_style.json");
        }
    }
    if let Some(id) = app.windows.get_id(WindowType::Settings) {
        app.windows.remove_id(WindowType::Settings);
        app.editing_color_index = None;
        return window::close(id);
    }
    Task::none()
}

/// Gère l'annulation des settings
pub fn handle_cancel_settings(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use iced::window;
    
    app.editing_style = None;
    app.editing_color_index = None;
    if let Some(id) = app.windows.get_id(WindowType::Settings) {
        app.windows.remove_id(WindowType::Settings);
        return window::close(id);
    }
    Task::none()
}

/// Gère le toggle du color picker
pub fn handle_toggle_color_picker(app: &mut ChartApp, index: usize) -> Task<crate::app::messages::Message> {
    if app.editing_color_index == Some(index) {
        app.editing_color_index = None;
    } else {
        app.editing_color_index = Some(index);
    }
    Task::none()
}

/// Gère le toggle de l'auto-scroll
pub fn handle_toggle_auto_scroll(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    if let Some(ref mut style) = app.editing_style {
        style.auto_scroll_enabled = !style.auto_scroll_enabled;
    }
    Task::none()
}

