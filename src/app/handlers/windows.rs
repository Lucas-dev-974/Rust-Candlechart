//! Handlers pour la gestion des fenêtres

use iced::{Task, Size, window};
use crate::app::app_state::ChartApp;
use crate::app::window_manager::WindowType;
use crate::app::utils::constants::{SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT};

/// Gère l'ouverture de la fenêtre de settings
pub fn handle_open_settings(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    
    if app.windows.is_open(WindowType::Settings) {
        return Task::none();
    }
    app.editing_style = Some(app.chart_style.clone());
    app.editing_color_index = None;
    
    let (id, task) = window::open(window::Settings {
        size: Size::new(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT),
        resizable: false,
        ..Default::default()
    });
    app.windows.set_id(WindowType::Settings, id);
    task.map(Message::SettingsWindowOpened)
}

/// Gère l'ouverture de la fenêtre de téléchargements
pub fn handle_open_downloads(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    
    if app.windows.is_open(WindowType::Downloads) {
        return Task::none();
    }
    
    let (id, task) = window::open(window::Settings {
        size: Size::new(500.0, 400.0),
        resizable: true,
        ..Default::default()
    });
    app.windows.set_id(WindowType::Downloads, id);
    task.map(Message::DownloadsWindowOpened)
}

/// Gère la fermeture d'une fenêtre
pub fn handle_window_closed(app: &mut ChartApp, id: window::Id) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    use iced::exit;
    
    match app.windows.get_window_type(id) {
        Some(WindowType::Settings) => {
            app.windows.remove_id(WindowType::Settings);
            app.editing_style = None;
            app.editing_color_index = None;
        }
        Some(WindowType::ProviderConfig) => {
            app.windows.remove_id(WindowType::ProviderConfig);
            app.editing_provider_token.clear();
        }
        Some(WindowType::Downloads) => {
            app.windows.remove_id(WindowType::Downloads);
        }
        Some(WindowType::Main) => {
            // Quitter l'application quand la fenêtre principale est fermée
            return exit();
        }
        None => {}
    }
    Task::none()
}

