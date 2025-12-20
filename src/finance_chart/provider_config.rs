//! Configuration des providers de données en temps réel
//!
//! Gère la sélection du provider actif et le stockage des tokens API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types de providers disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    Binance,
    // Ajouter d'autres providers ici à l'avenir
    // Coinbase,
    // Kraken,
    // etc.
}

impl ProviderType {
    /// Retourne tous les providers disponibles
    pub fn all() -> Vec<ProviderType> {
        vec![ProviderType::Binance]
    }

    /// Retourne le nom d'affichage du provider
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderType::Binance => "Binance",
        }
    }

    /// Retourne la description du provider
    pub fn description(&self) -> &'static str {
        match self {
            ProviderType::Binance => "API publique Binance pour les données de marché en temps réel",
        }
    }
}

/// Configuration d'un provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Type de provider
    pub provider_type: ProviderType,
    /// Token API (optionnel, certains providers n'en ont pas besoin)
    pub api_token: Option<String>,
    /// Clé secrète API (optionnel, pour certains providers)
    pub api_secret: Option<String>,
}

impl ProviderConfig {
    pub fn new(provider_type: ProviderType) -> Self {
        Self {
            provider_type,
            api_token: None,
            api_secret: None,
        }
    }

    pub fn with_token(provider_type: ProviderType, api_token: String) -> Self {
        Self {
            provider_type,
            api_token: Some(api_token),
            api_secret: None,
        }
    }
}

/// Gestionnaire de configuration des providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigManager {
    /// Provider actuellement sélectionné
    pub active_provider: ProviderType,
    /// Configurations de tous les providers
    pub providers: HashMap<ProviderType, ProviderConfig>,
}

impl Default for ProviderConfigManager {
    fn default() -> Self {
        let mut manager = Self {
            active_provider: ProviderType::Binance,
            providers: HashMap::new(),
        };

        // Initialiser avec les providers par défaut
        for provider_type in ProviderType::all() {
            manager.providers.insert(
                provider_type,
                ProviderConfig::new(provider_type),
            );
        }

        manager
    }
}

impl ProviderConfigManager {
    /// Crée un nouveau gestionnaire de configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Charge la configuration depuis un fichier
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let manager: ProviderConfigManager = serde_json::from_str(&json)?;
        Ok(manager)
    }

    /// Sauvegarde la configuration dans un fichier
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Retourne la configuration du provider actif
    pub fn active_config(&self) -> Option<&ProviderConfig> {
        self.providers.get(&self.active_provider)
    }

    /// Met à jour la configuration d'un provider
    pub fn update_provider_config(&mut self, provider_type: ProviderType, config: ProviderConfig) {
        self.providers.insert(provider_type, config);
    }

    /// Met à jour le token API d'un provider
    pub fn set_provider_token(&mut self, provider_type: ProviderType, token: Option<String>) {
        let config = self.providers.entry(provider_type).or_insert_with(|| {
            ProviderConfig::new(provider_type)
        });
        config.api_token = token;
    }

    /// Met à jour la clé secrète API d'un provider
    pub fn set_provider_secret(&mut self, provider_type: ProviderType, secret: Option<String>) {
        let config = self.providers.entry(provider_type).or_insert_with(|| {
            ProviderConfig::new(provider_type)
        });
        config.api_secret = secret;
    }

    /// Change le provider actif
    pub fn set_active_provider(&mut self, provider_type: ProviderType) {
        self.active_provider = provider_type;
    }

    /// Retourne tous les providers disponibles
    pub fn available_providers(&self) -> Vec<ProviderType> {
        ProviderType::all()
    }
}

