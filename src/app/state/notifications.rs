//! Système de notifications pour l'application
//!
//! Ce module gère l'affichage des notifications (erreurs, avertissements, succès, info)
//! avec support pour les notifications temporaires et persistantes.

use std::time::{Duration, Instant};
use crate::app::error_handling::{AppError, ErrorType};

/// Type de notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// Erreur (rouge)
    Error,
    /// Avertissement (orange)
    Warning,
    /// Succès (vert)
    Success,
    /// Information (bleu)
    Info,
}

/// Notification affichée à l'utilisateur
#[derive(Debug, Clone)]
pub struct Notification {
    /// Identifiant unique de la notification
    pub id: usize,
    /// Message principal pour l'utilisateur
    pub message: String,
    /// Message détaillé (optionnel)
    pub details: Option<String>,
    /// Type de notification
    pub notification_type: NotificationType,
    /// Timestamp de création
    pub created_at: Instant,
    /// Durée avant auto-dismiss (None = persistant)
    pub auto_dismiss_duration: Option<Duration>,
    /// Source de la notification (optionnel)
    pub source: Option<String>,
}

impl Notification {
    /// Crée une nouvelle notification
    pub fn new(
        id: usize,
        message: String,
        notification_type: NotificationType,
    ) -> Self {
        Self {
            id,
            message,
            details: None,
            notification_type,
            created_at: Instant::now(),
            auto_dismiss_duration: None,
            source: None,
        }
    }

    /// Crée une notification avec détails
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Crée une notification temporaire (auto-dismiss après la durée spécifiée)
    pub fn with_auto_dismiss(mut self, duration: Duration) -> Self {
        self.auto_dismiss_duration = Some(duration);
        self
    }

    /// Ajoute une source à la notification
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Vérifie si la notification doit être auto-dismissée
    pub fn should_auto_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss_duration {
            self.created_at.elapsed() >= duration
        } else {
            false
        }
    }

    /// Convertit une AppError en Notification
    pub fn from_error(id: usize, error: AppError) -> Self {
        let notification_type = match error.error_type {
            ErrorType::Network | ErrorType::Api => NotificationType::Error,
            ErrorType::Validation => NotificationType::Warning,
            ErrorType::Parse => NotificationType::Warning,
            ErrorType::Configuration => NotificationType::Error,
            ErrorType::Unknown => NotificationType::Warning,
        };

        let user_message = error.user_message.clone();
        let technical_message = error.technical_message.clone();

        Self {
            id,
            message: user_message.clone(),
            details: if technical_message != user_message {
                Some(technical_message)
            } else {
                None
            },
            notification_type,
            created_at: error.timestamp,
            auto_dismiss_duration: None, // Les erreurs sont persistantes par défaut
            source: error.source,
        }
    }
}

/// Gestionnaire de notifications
#[derive(Debug, Clone)]
pub struct NotificationManager {
    /// Liste des notifications actives
    notifications: Vec<Notification>,
    /// Compteur pour générer des IDs uniques
    next_id: usize,
    /// Nombre maximum de notifications affichées simultanément
    max_notifications: usize,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            next_id: 0,
            max_notifications: 5,
        }
    }
}

impl NotificationManager {
    /// Crée un nouveau gestionnaire de notifications
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée un gestionnaire avec une limite personnalisée
    pub fn with_max_notifications(max: usize) -> Self {
        Self {
            max_notifications: max,
            ..Default::default()
        }
    }

    /// Ajoute une notification
    pub fn add(&mut self, mut notification: Notification) {
        // Assigner un ID si nécessaire
        if notification.id == 0 {
            notification.id = self.next_id;
            self.next_id += 1;
        }

        // Limiter le nombre de notifications
        if self.notifications.len() >= self.max_notifications {
            self.notifications.remove(0);
        }

        self.notifications.push(notification);
    }

    /// Supprime une notification par ID
    pub fn remove(&mut self, id: usize) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Supprime une notification par index
    pub fn remove_by_index(&mut self, index: usize) {
        if index < self.notifications.len() {
            self.notifications.remove(index);
        }
    }

    /// Supprime toutes les notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Supprime toutes les notifications d'un type donné
    pub fn clear_by_type(&mut self, notification_type: NotificationType) {
        self.notifications.retain(|n| n.notification_type != notification_type);
    }

    /// Récupère toutes les notifications
    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Vérifie et supprime les notifications expirées
    pub fn update(&mut self) -> Vec<usize> {
        let mut expired_ids = Vec::new();
        
        let mut i = 0;
        while i < self.notifications.len() {
            if self.notifications[i].should_auto_dismiss() {
                expired_ids.push(self.notifications[i].id);
                self.notifications.remove(i);
            } else {
                i += 1;
            }
        }

        expired_ids
    }

    /// Ajoute une notification d'erreur depuis une AppError
    pub fn add_error(&mut self, error: AppError) {
        let notification = Notification::from_error(self.next_id, error);
        self.add(notification);
    }

    /// Ajoute une notification de succès
    pub fn add_success(&mut self, message: String) {
        let notification = Notification::new(self.next_id, message, NotificationType::Success)
            .with_auto_dismiss(Duration::from_secs(3));
        self.add(notification);
    }

    /// Ajoute une notification d'avertissement
    pub fn add_warning(&mut self, message: String) {
        let notification = Notification::new(self.next_id, message, NotificationType::Warning)
            .with_auto_dismiss(Duration::from_secs(5));
        self.add(notification);
    }

    /// Ajoute une notification d'information
    pub fn add_info(&mut self, message: String) {
        let notification = Notification::new(self.next_id, message, NotificationType::Info)
            .with_auto_dismiss(Duration::from_secs(4));
        self.add(notification);
    }
}
