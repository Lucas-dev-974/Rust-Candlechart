//! Handlers pour la gestion de la configuration des providers

use iced::{Task, Size, window};
use crate::app::app_state::ChartApp;
use crate::app::window_manager::WindowType;
use crate::finance_chart::{ProviderType, BinanceProvider};
use std::sync::Arc;

/// Gère l'ouverture de la fenêtre de configuration des providers
pub fn handle_open_provider_config(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    
    if app.windows.is_open(WindowType::ProviderConfig) {
        return Task::none();
    }
    
    // Initialiser les tokens en cours d'édition
    for provider_type in ProviderType::all() {
        if let Some(config) = app.provider_config.providers.get(&provider_type) {
            app.editing_provider_token.insert(
                provider_type,
                config.api_token.clone().unwrap_or_default(),
            );
        } else {
            app.editing_provider_token.insert(provider_type, String::new());
        }
    }
    
    let (id, task) = window::open(window::Settings {
        size: Size::new(600.0, 500.0),
        resizable: false,
        ..Default::default()
    });
    app.windows.set_id(WindowType::ProviderConfig, id);
    task.map(Message::ProviderConfigWindowOpened)
}

/// Gère la mise à jour du token d'un provider
pub fn handle_update_provider_token(
    app: &mut ChartApp,
    provider_type: ProviderType,
    token: String
) -> Task<crate::app::messages::Message> {
    app.editing_provider_token.insert(provider_type, token);
    Task::none()
}

/// Gère l'application de la configuration des providers
pub fn handle_apply_provider_config(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use iced::window;
    
    // Appliquer les tokens modifiés
    for (provider_type, token) in &app.editing_provider_token {
        let token_opt = if token.is_empty() {
            None
        } else {
            Some(token.clone())
        };
        app.provider_config.set_provider_token(*provider_type, token_opt);
    }
    
    // Sauvegarder la configuration
    if let Err(e) = app.provider_config.save_to_file("provider_config.json") {
        eprintln!("⚠️ Erreur sauvegarde configuration providers: {}", e);
    } else {
        println!("✅ Configuration des providers sauvegardée dans provider_config.json");
    }
    
    // Recréer le provider avec la nouvelle configuration (Arc pour partage efficace)
    if let Some(config) = app.provider_config.active_config() {
        app.binance_provider = Arc::new(BinanceProvider::with_token(config.api_token.clone()));
        println!("✅ Provider recréé avec la nouvelle configuration");
    }
    
    // Fermer la fenêtre
    if let Some(id) = app.windows.get_id(WindowType::ProviderConfig) {
        app.windows.remove_id(WindowType::ProviderConfig);
        app.editing_provider_token.clear();
        return window::close(id);
    }
    Task::none()
}

/// Gère la sélection d'un provider
pub fn handle_select_provider(app: &mut ChartApp, provider_type: ProviderType) -> Task<crate::app::messages::Message> {
    app.provider_config.set_active_provider(provider_type);
    
    // Recréer le provider avec la configuration du nouveau provider actif (Arc pour partage efficace)
    if let Some(config) = app.provider_config.active_config() {
        app.binance_provider = Arc::new(BinanceProvider::with_token(config.api_token.clone()));
        println!("✅ Provider changé et recréé");
    }
    
    Task::none()
}

/// Gère l'annulation de la configuration des providers
pub fn handle_cancel_provider_config(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    use iced::window;
    
    if let Some(id) = app.windows.get_id(WindowType::ProviderConfig) {
        app.windows.remove_id(WindowType::ProviderConfig);
        app.editing_provider_token.clear();
        return window::close(id);
    }
    Task::none()
}

/// Gère le test de connexion au provider
pub fn handle_test_provider_connection(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.provider_connection_testing = true;
    app.provider_connection_status = None;
    crate::app::realtime::test_provider_connection(app)
}

/// Gère le résultat du test de connexion
pub fn handle_provider_connection_test_complete(
    app: &mut ChartApp,
    result: Result<(), String>
) -> Task<crate::app::messages::Message> {
    app.provider_connection_testing = false;
    app.provider_connection_status = Some(result.is_ok());
    if let Err(e) = &result {
        eprintln!("❌ Test de connexion échoué: {}", e);
    } else {
        println!("✅ Connexion au provider réussie");
    }
    Task::none()
}

