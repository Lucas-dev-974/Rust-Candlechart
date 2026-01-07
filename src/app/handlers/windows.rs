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

/// Gère l'ouverture de la fenêtre des actifs
pub fn handle_open_assets(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    use crate::app::realtime::load_assets;
    use crate::app::persistence::AssetsPersistenceState;
    
    if app.windows.is_open(WindowType::Assets) {
        return Task::none();
    }
    
    let (id, task) = window::open(window::Settings {
        size: Size::new(800.0, 600.0),
        resizable: true,
        ..Default::default()
    });
    app.windows.set_id(WindowType::Assets, id);
    
    // Charger les actifs depuis le fichier JSON d'abord
    if let Ok(persistence_state) = AssetsPersistenceState::load_from_file("assets.json") {
        println!("📂 Chargement des actifs depuis le fichier: {} actifs", persistence_state.assets.len());
        app.assets = persistence_state.assets.clone();
    } else {
        println!("ℹ️ Aucun fichier d'actifs trouvé, chargement depuis le provider...");
    }
    
    // Vérifier en arrière-plan s'il y a de nouveaux actifs disponibles
    Task::batch(vec![
        task.map(Message::AssetsWindowOpened),
        load_assets(app),
    ])
}

/// Gère la fermeture d'une fenêtre
pub fn handle_window_closed(app: &mut ChartApp, id: window::Id) -> Task<crate::app::messages::Message> {
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
            app.editing_provider_secret.clear();
        }
        Some(WindowType::Downloads) => {
            app.windows.remove_id(WindowType::Downloads);
        }
        Some(WindowType::Assets) => {
            app.windows.remove_id(WindowType::Assets);
        }
        Some(WindowType::Main) => {
            // Quitter l'application quand la fenêtre principale est fermée
            return exit();
        }
        None => {}
    }
    Task::none()
}

