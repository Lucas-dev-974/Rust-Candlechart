//! Handlers pour la gestion des notifications

use crate::app::app_state::ChartApp;
use crate::app::messages::Message;
use crate::app::state::notifications::Notification;
use iced::Task;

/// Gère l'affichage d'une notification
pub fn handle_show_notification(app: &mut ChartApp, notification: Notification) -> Task<Message> {
    app.ui.notifications.add(notification);
    Task::none()
}

/// Gère la fermeture d'une notification par ID
pub fn handle_dismiss_notification(app: &mut ChartApp, id: usize) -> Task<Message> {
    app.ui.notifications.remove(id);
    Task::none()
}

/// Gère la fermeture de toutes les notifications
pub fn handle_clear_all_notifications(app: &mut ChartApp) -> Task<Message> {
    app.ui.notifications.clear();
    Task::none()
}

/// Met à jour les notifications (supprime les expirées)
pub fn handle_update_notifications(app: &mut ChartApp) -> Task<Message> {
    let _expired_ids = app.ui.update_notifications();
    
    // Si des notifications ont expiré, on pourrait déclencher un refresh
    // mais pour l'instant on retourne juste Task::none()
    // L'UI sera mise à jour au prochain cycle de rendu
    
    Task::none()
}

/// Affiche une notification de succès
pub fn show_success_notification(app: &mut ChartApp, message: String) -> Task<Message> {
    app.ui.notifications.add_success(message);
    Task::none()
}

/// Affiche une notification d'avertissement
pub fn show_warning_notification(app: &mut ChartApp, message: String) -> Task<Message> {
    app.ui.notifications.add_warning(message);
    Task::none()
}

/// Affiche une notification d'information
pub fn show_info_notification(app: &mut ChartApp, message: String) -> Task<Message> {
    app.ui.notifications.add_info(message);
    Task::none()
}

/// Génère une notification de succès de test
pub fn handle_test_success_notification(app: &mut ChartApp) -> Task<Message> {
    app.ui.notifications.add_success(
        "Opération réussie ! Ceci est une notification de succès.".to_string()
    );
    Task::none()
}

/// Génère une notification d'info de test
pub fn handle_test_info_notification(app: &mut ChartApp) -> Task<Message> {
    app.ui.notifications.add_info(
        "Information : Ceci est une notification d'information.".to_string()
    );
    Task::none()
}
