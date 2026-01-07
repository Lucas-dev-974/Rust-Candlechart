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
use crate::finance_chart::BinanceProvider;

/// Teste la connexion au provider actif
pub fn test_provider_connection(app: &ChartApp) -> Task<Message> {
    // Récupérer le token en cours d'édition pour le provider actif
    let editing_token = app.editing_provider_token
        .get(&app.provider_config.active_provider)
        .cloned()
        .filter(|t| !t.is_empty());
    
    // Si pas de token en cours d'édition, utiliser celui de la config sauvegardée
    let token_to_test = editing_token.or_else(|| {
        app.provider_config
            .active_config()
            .and_then(|c| c.api_token.clone())
    });
    
    let has_token = token_to_test.is_some();
    
    println!("🔍 Test de connexion au provider...");
    if has_token {
        println!("   Utilisation du token API pour le test");
    } else {
        println!("   Test de connexion de base (sans authentification)");
    }
    
    // Créer un provider temporaire avec le token à tester
    let test_provider = Arc::new(BinanceProvider::with_token(token_to_test.clone()));
    
    Task::perform(
        async move {
            // Si un token est configuré, tester l'authentification
            // Sinon, tester juste la connexion de base
            if has_token {
                test_provider.test_authenticated_connection().await
                    .map_err(|e| e.to_string())
            } else {
                test_provider.test_connection().await
                    .map_err(|e| e.to_string())
            }
        },
        Message::ProviderConnectionTestComplete,
    )
}

/// Récupère les informations du compte depuis le provider
pub fn fetch_account_info(app: &ChartApp) -> Task<Message> {
    // Récupérer le token et la clé secrète depuis la config
    let token = app.provider_config
        .active_config()
        .and_then(|c| c.api_token.clone());
    
    let secret = app.provider_config
        .active_config()
        .and_then(|c| c.api_secret.clone());
    
    if token.is_none() {
        return Task::perform(
            async move {
                Err("Aucun token API configuré".to_string())
            },
            Message::AccountInfoFetched,
        );
    }
    
    if secret.is_none() {
        return Task::perform(
            async move {
                Err("Aucune clé secrète API configurée. Veuillez configurer votre clé secrète pour récupérer les informations du compte.".to_string())
            },
            Message::AccountInfoFetched,
        );
    }
    
    // Créer un provider temporaire avec le token et la clé secrète
    let provider = Arc::new(BinanceProvider::with_token_and_secret(token, secret));
    
    println!("🔍 Récupération des informations du compte...");
    println!("   Clé secrète disponible, génération de la signature HMAC");
    
    Task::perform(
        async move {
            provider.get_account_info().await
                .map_err(|e| e.to_string())
        },
        Message::AccountInfoFetched,
    )
}

