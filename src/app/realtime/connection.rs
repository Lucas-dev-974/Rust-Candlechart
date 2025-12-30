//! Test de connexion au provider
//!
//! Ce module gère les tests de connexion et d'authentification
//! avec les providers de données.

use iced::Task;
use std::sync::Arc;
use crate::app::{
    messages::Message,
    app_state::ChartApp,
};

/// Teste la connexion au provider actif
pub fn test_provider_connection(app: &ChartApp) -> Task<Message> {
    let provider = Arc::clone(&app.binance_provider);
    let has_token = app.provider_config
        .active_config()
        .map(|c| c.api_token.is_some())
        .unwrap_or(false);
    
    println!("🔍 Test de connexion au provider...");
    
    Task::perform(
        async move {
            // Si un token est configuré, tester l'authentification
            // Sinon, tester juste la connexion de base
            if has_token {
                provider.test_authenticated_connection().await
                    .map_err(|e| e.to_string())
            } else {
                provider.test_connection().await
                    .map_err(|e| e.to_string())
            }
        },
        Message::ProviderConnectionTestComplete,
    )
}

