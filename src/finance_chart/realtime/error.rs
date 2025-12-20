//! Types d'erreur pour les providers de données en temps réel

use std::fmt;

/// Erreur lors de la récupération de données depuis un provider
#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Erreur de réseau (connexion, timeout, etc.)
    Network(String),
    /// Erreur de parsing (JSON, format de données, etc.)
    Parse(String),
    /// Erreur d'API (code HTTP d'erreur, message d'erreur API)
    Api {
        status: Option<u16>,
        message: String,
    },
    /// Format de SeriesId invalide
    InvalidSeriesId(String),
    /// Erreur de validation des données
    Validation(String),
    /// Erreur inconnue
    Unknown(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Network(msg) => write!(f, "Erreur réseau: {}", msg),
            ProviderError::Parse(msg) => write!(f, "Erreur de parsing: {}", msg),
            ProviderError::Api { status, message } => {
                if let Some(status) = status {
                    write!(f, "Erreur API ({}): {}", status, message)
                } else {
                    write!(f, "Erreur API: {}", message)
                }
            }
            ProviderError::InvalidSeriesId(msg) => write!(f, "Format SeriesId invalide: {}", msg),
            ProviderError::Validation(msg) => write!(f, "Erreur de validation: {}", msg),
            ProviderError::Unknown(msg) => write!(f, "Erreur inconnue: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Network(format!("Timeout: {}", err))
        } else if err.is_connect() {
            ProviderError::Network(format!("Erreur de connexion: {}", err))
        } else {
            ProviderError::Network(format!("Erreur HTTP: {}", err))
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::Parse(format!("Erreur JSON: {}", err))
    }
}

impl From<String> for ProviderError {
    fn from(msg: String) -> Self {
        ProviderError::Unknown(msg)
    }
}

impl From<&str> for ProviderError {
    fn from(msg: &str) -> Self {
        ProviderError::Unknown(msg.to_string())
    }
}

