//! Module de gestion d'erreurs centralisé
//!
//! Ce module fournit:
//! - Types d'erreurs améliorés avec contexte
//! - Système de retry avec backoff exponentiel
//! - Logging structuré
//! - Messages d'erreur utilisateur-friendly

use crate::finance_chart::realtime::ProviderError;
use log::{error, warn, info, debug};
use std::time::{Duration, Instant};
use std::fmt;

/// Erreur utilisateur-friendly avec contexte
#[derive(Debug, Clone)]
pub struct AppError {
    /// Message principal pour l'utilisateur
    pub user_message: String,
    /// Message technique pour le debugging
    pub technical_message: String,
    /// Type d'erreur
    pub error_type: ErrorType,
    /// Erreur source (si disponible)
    pub source: Option<String>,
    /// Timestamp de l'erreur
    pub timestamp: Instant,
}

/// Type d'erreur pour catégorisation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Erreur réseau (connexion, timeout)
    Network,
    /// Erreur d'API (rate limit, authentification, etc.)
    Api,
    /// Erreur de validation (données invalides)
    Validation,
    /// Erreur de parsing (JSON, format)
    Parse,
    /// Erreur de configuration
    Configuration,
    /// Erreur inconnue
    Unknown,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)
    }
}

impl std::error::Error for AppError {}

impl AppError {
    /// Crée une nouvelle erreur avec message utilisateur-friendly
    pub fn new(user_message: String, technical_message: String, error_type: ErrorType) -> Self {
        Self {
            user_message,
            technical_message,
            error_type,
            source: None,
            timestamp: Instant::now(),
        }
    }

    /// Ajoute une source à l'erreur
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Convertit une ProviderError en AppError
    pub fn from_provider_error(err: ProviderError, context: &str) -> Self {
        match err {
            ProviderError::Network(msg) => {
                let user_msg = format!("Problème de connexion réseau lors de {}", context);
                Self::new(
                    user_msg,
                    format!("Network error: {}", msg),
                    ErrorType::Network,
                )
            }
            ProviderError::Parse(msg) => {
                let user_msg = format!("Erreur lors du traitement des données de {}", context);
                Self::new(
                    user_msg,
                    format!("Parse error: {}", msg),
                    ErrorType::Parse,
                )
            }
            ProviderError::Api { status, message } => {
                let user_msg = if let Some(status) = status {
                    if status == 429 {
                        "Trop de requêtes. Veuillez patienter quelques instants.".to_string()
                    } else if status == 401 || status == 403 {
                        "Erreur d'authentification. Vérifiez vos clés API.".to_string()
                    } else {
                        format!("Erreur API ({}): {}", status, message)
                    }
                } else {
                    format!("Erreur API: {}", message)
                };
                Self::new(
                    user_msg,
                    format!("API error (status: {:?}): {}", status, message),
                    ErrorType::Api,
                )
            }
            ProviderError::InvalidSeriesId(msg) => {
                Self::new(
                    format!("Identifiant de série invalide"),
                    format!("Invalid series ID: {}", msg),
                    ErrorType::Validation,
                )
            }
            ProviderError::Validation(msg) => {
                Self::new(
                    format!("Données invalides"),
                    format!("Validation error: {}", msg),
                    ErrorType::Validation,
                )
            }
            ProviderError::Unknown(msg) => {
                Self::new(
                    format!("Erreur inconnue lors de {}", context),
                    format!("Unknown error: {}", msg),
                    ErrorType::Unknown,
                )
            }
        }
    }

    /// Log l'erreur avec le niveau approprié
    pub fn log(&self) {
        let log_msg = if let Some(ref source) = self.source {
            format!("[{}] {} - {}", source, self.user_message, self.technical_message)
        } else {
            format!("{} - {}", self.user_message, self.technical_message)
        };

        match self.error_type {
            ErrorType::Network | ErrorType::Api => error!("{}", log_msg),
            ErrorType::Validation => warn!("{}", log_msg),
            ErrorType::Parse => warn!("{}", log_msg),
            ErrorType::Configuration => error!("{}", log_msg),
            ErrorType::Unknown => error!("{}", log_msg),
        }
    }
}

/// Configuration pour le système de retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Nombre maximum de tentatives
    pub max_attempts: u32,
    /// Délai initial en millisecondes
    pub initial_delay_ms: u64,
    /// Facteur de multiplication pour le backoff exponentiel
    pub backoff_multiplier: f64,
    /// Délai maximum entre les tentatives en millisecondes
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
        }
    }
}

impl RetryConfig {
    /// Configuration pour les requêtes API critiques (plus de tentatives)
    pub fn for_api_calls() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 500,
            backoff_multiplier: 2.0,
            max_delay_ms: 5000,
        }
    }

    /// Configuration pour les requêtes non-critiques (moins de tentatives)
    pub fn for_non_critical() -> Self {
        Self {
            max_attempts: 2,
            initial_delay_ms: 1000,
            backoff_multiplier: 1.5,
            max_delay_ms: 3000,
        }
    }
}

/// Résultat d'une opération avec retry
#[derive(Debug)]
pub enum RetryResult<T> {
    /// Succès après N tentatives
    Success { value: T, attempts: u32 },
    /// Échec après toutes les tentatives
    Failed { error: AppError, attempts: u32 },
}

/// Exécute une fonction avec retry automatique
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
    context: &str,
) -> RetryResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: Into<AppError>,
{
    let mut delay_ms = config.initial_delay_ms;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        debug!("Tentative {}/{} pour {}", attempt, config.max_attempts, context);

        match operation().await {
            Ok(value) => {
                if attempt > 1 {
                    info!("Succès après {} tentatives pour {}", attempt, context);
                }
                return RetryResult::Success {
                    value,
                    attempts: attempt,
                };
            }
            Err(e) => {
                let app_error: AppError = e.into();
                last_error = Some(app_error.clone());
                
                // Log l'erreur
                app_error.log();

                // Ne pas retry pour certaines erreurs
                if should_not_retry(&app_error) {
                    warn!("Erreur non-retryable détectée, arrêt des tentatives");
                    return RetryResult::Failed {
                        error: app_error,
                        attempts: attempt,
                    };
                }

                // Si ce n'est pas la dernière tentative, attendre avant de réessayer
                if attempt < config.max_attempts {
                    let delay = Duration::from_millis(delay_ms.min(config.max_delay_ms));
                    debug!("Attente de {:?} avant la prochaine tentative", delay);
                    tokio::time::sleep(delay).await;
                    
                    // Calculer le délai pour la prochaine tentative (backoff exponentiel)
                    delay_ms = ((delay_ms as f64) * config.backoff_multiplier) as u64;
                }
            }
        }
    }

    // Toutes les tentatives ont échoué
    let final_error = last_error.unwrap_or_else(|| {
        AppError::new(
            format!("Échec après {} tentatives", config.max_attempts),
            format!("Toutes les tentatives ont échoué pour {}", context),
            ErrorType::Unknown,
        )
    });

    error!("Échec définitif après {} tentatives pour {}", config.max_attempts, context);
    RetryResult::Failed {
        error: final_error,
        attempts: config.max_attempts,
    }
}

/// Détermine si une erreur ne doit pas être retentée
fn should_not_retry(error: &AppError) -> bool {
    match error.error_type {
        // Ne pas retry pour les erreurs de validation ou de configuration
        ErrorType::Validation | ErrorType::Configuration => true,
        // Ne pas retry pour certaines erreurs API spécifiques
        ErrorType::Api => {
            // Ne pas retry pour les erreurs d'authentification (401, 403)
            error.technical_message.contains("401") || 
            error.technical_message.contains("403") ||
            error.technical_message.contains("INVALID_API_KEY") ||
            error.technical_message.contains("INVALID_SIGNATURE")
        }
        _ => false,
    }
}

/// Helper pour convertir une String en AppError
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::new(
            msg.clone(),
            msg,
            ErrorType::Unknown,
        )
    }
}

/// Helper pour convertir une &str en AppError
impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::new(
            msg.to_string(),
            msg.to_string(),
            ErrorType::Unknown,
        )
    }
}
