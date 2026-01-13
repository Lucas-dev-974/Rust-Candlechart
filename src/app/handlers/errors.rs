//! Handlers pour la gestion des erreurs

use crate::app::app_state::ChartApp;
use crate::app::messages::Message;
use iced::Task;

/// Gère l'affichage d'un message d'erreur
pub fn handle_show_error(app: &mut ChartApp, error: crate::app::error_handling::AppError) -> Task<Message> {
    // Log l'erreur
    error.log();
    
    // Ajouter l'erreur à l'UI (ancien système pour compatibilité)
    app.ui.add_error(error.clone());
    
    // Ajouter aussi dans le système de notifications
    app.ui.notifications.add_error(error);
    
    Task::none()
}

/// Gère la fermeture d'un message d'erreur
pub fn handle_dismiss_error(app: &mut ChartApp, index: usize) -> Task<Message> {
    app.ui.remove_error(index);
    Task::none()
}

/// Gère la fermeture de tous les messages d'erreur
pub fn handle_clear_all_errors(app: &mut ChartApp) -> Task<Message> {
    app.ui.clear_errors();
    Task::none()
}

/// Génère une erreur de test pour vérifier l'implémentation de la gestion d'erreurs
pub fn handle_test_error(app: &mut ChartApp) -> Task<Message> {
    use crate::app::error_handling::{AppError, ErrorType};
    
    // Générer une erreur de test avec différents types pour tester l'affichage
    let test_error = AppError::new(
        "Erreur de test - Ceci est un message utilisateur-friendly".to_string(),
        "Technical error: This is a test error to verify error handling implementation".to_string(),
        ErrorType::Api,
    )
    .with_source("Test".to_string());
    
    // Log et ajouter l'erreur
    test_error.log();
    // Utiliser uniquement le nouveau système de notifications pour éviter les doublons
    app.ui.notifications.add_error(test_error);
    
    Task::none()
}
