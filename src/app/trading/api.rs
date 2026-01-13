//! Module API pour le trading réel via Binance
//!
//! Ce module gère les appels API pour placer des ordres réels sur Binance,
//! avec validation, gestion d'erreurs et suivi des ordres.

use crate::finance_chart::providers::binance::BinanceProvider;
use crate::finance_chart::realtime::ProviderError;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Résultat d'un placement d'ordre
#[derive(Debug, Clone)]
pub enum OrderResult {
    /// Ordre placé avec succès
    Success(OrderResponse),
    /// Erreur lors du placement
    Error(String),
}

/// Réponse de l'API Binance pour un ordre
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// ID de l'ordre côté Binance
    #[serde(rename = "orderId")]
    pub order_id: u64,
    /// ID client de l'ordre (si fourni)
    #[serde(rename = "clientOrderId", default)]
    pub client_order_id: Option<String>,
    /// Symbole tradé
    pub symbol: String,
    /// Type d'ordre (MARKET, LIMIT, etc.)
    #[serde(rename = "type")]
    pub order_type: String,
    /// Côté de l'ordre (BUY, SELL)
    pub side: String,
    /// Quantité
    pub quantity: String,
    /// Prix (pour les ordres LIMIT)
    #[serde(default)]
    pub price: Option<String>,
    /// Prix d'exécution moyen
    #[serde(rename = "executedQty", default)]
    pub executed_quantity: Option<String>,
    /// Statut de l'ordre (NEW, FILLED, PARTIALLY_FILLED, etc.)
    pub status: String,
    /// Timestamp de l'ordre
    pub time: i64,
}

/// Erreur de validation d'ordre
#[derive(Debug, Clone)]
pub enum OrderValidationError {
    /// Quantité invalide ou nulle
    InvalidQuantity,
    /// Prix invalide ou nul (pour les ordres LIMIT)
    InvalidPrice,
    /// Solde insuffisant
    InsufficientBalance { required: f64, available: f64 },
    /// Symbole invalide
    InvalidSymbol,
    /// Configuration API manquante
    MissingApiConfig,
}

impl std::fmt::Display for OrderValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderValidationError::InvalidQuantity => {
                write!(f, "Quantité invalide ou nulle")
            }
            OrderValidationError::InvalidPrice => {
                write!(f, "Prix invalide ou nul (requis pour les ordres LIMIT)")
            }
            OrderValidationError::InsufficientBalance { required, available } => {
                write!(
                    f,
                    "Solde insuffisant: {:.2} requis, {:.2} disponible",
                    required, available
                )
            }
            OrderValidationError::InvalidSymbol => {
                write!(f, "Symbole de trading invalide")
            }
            OrderValidationError::MissingApiConfig => {
                write!(f, "Configuration API manquante (token et secret requis)")
            }
        }
    }
}

/// Valide un ordre avant de le placer
pub fn validate_order(
    symbol: &str,
    quantity: f64,
    price: Option<f64>,
    order_type: &str,
    available_balance: f64,
) -> Result<(), OrderValidationError> {
    // Vérifier le symbole
    if symbol.is_empty() {
        return Err(OrderValidationError::InvalidSymbol);
    }

    // Vérifier la quantité
    if quantity <= 0.0 {
        return Err(OrderValidationError::InvalidQuantity);
    }

    // Vérifier le prix pour les ordres LIMIT
    if order_type == "LIMIT" {
        if let Some(p) = price {
            if p <= 0.0 {
                return Err(OrderValidationError::InvalidPrice);
            }
        } else {
            return Err(OrderValidationError::InvalidPrice);
        }
    }

    // Calculer le montant requis
    let required_amount = match order_type {
        "MARKET" => {
            // Pour les ordres MARKET, on ne connaît pas le prix exact
            // On utilise le prix fourni comme estimation ou on accepte sans vérification stricte
            // (Binance vérifiera de toute façon)
            if let Some(p) = price {
                quantity * p
            } else {
                // Pas de vérification stricte pour MARKET sans prix
                return Ok(());
            }
        }
        "LIMIT" => {
            if let Some(p) = price {
                quantity * p
            } else {
                return Err(OrderValidationError::InvalidPrice);
            }
        }
        _ => {
            // Type d'ordre non supporté
            return Ok(());
        }
    };

    // Vérifier le solde disponible
    if required_amount > available_balance {
        return Err(OrderValidationError::InsufficientBalance {
            required: required_amount,
            available: available_balance,
        });
    }

    Ok(())
}

/// Place un ordre Market BUY sur Binance
pub async fn place_market_buy_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
) -> Result<OrderResponse, ProviderError> {
    place_order(
        provider,
        symbol,
        "BUY",
        "MARKET",
        Some(quantity),
        None,
        None,
    )
    .await
}

/// Place un ordre Market SELL sur Binance
pub async fn place_market_sell_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
) -> Result<OrderResponse, ProviderError> {
    place_order(
        provider,
        symbol,
        "SELL",
        "MARKET",
        Some(quantity),
        None,
        None,
    )
    .await
}

/// Place un ordre Limit BUY sur Binance
pub async fn place_limit_buy_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
    price: f64,
    time_in_force: Option<&str>,
) -> Result<OrderResponse, ProviderError> {
    place_order(
        provider,
        symbol,
        "BUY",
        "LIMIT",
        Some(quantity),
        Some(price),
        time_in_force,
    )
    .await
}

/// Place un ordre Limit SELL sur Binance
pub async fn place_limit_sell_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
    price: f64,
    time_in_force: Option<&str>,
) -> Result<OrderResponse, ProviderError> {
    place_order(
        provider,
        symbol,
        "SELL",
        "LIMIT",
        Some(quantity),
        Some(price),
        time_in_force,
    )
    .await
}

/// Fonction générique pour placer un ordre sur Binance
async fn place_order(
    provider: &BinanceProvider,
    symbol: &str,
    side: &str,
    order_type: &str,
    quantity: Option<f64>,
    price: Option<f64>,
    time_in_force: Option<&str>,
) -> Result<OrderResponse, ProviderError> {
    // Vérifier que le provider a les credentials nécessaires
    if provider.api_token.is_none() || provider.api_secret.is_none() {
        return Err(ProviderError::Api {
            status: None,
            message: "Configuration API manquante (token et secret requis)".to_string(),
        });
    }

    // Générer le timestamp en millisecondes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ProviderError::Api {
            status: None,
            message: format!("Erreur génération timestamp: {}", e),
        })?
        .as_millis() as u64;

    // Construire les paramètres de la requête
    let mut params = vec![
        format!("symbol={}", symbol.to_uppercase()),
        format!("side={}", side),
        format!("type={}", order_type),
        format!("timestamp={}", timestamp),
    ];

    // Ajouter les paramètres selon le type d'ordre
    match order_type {
        "MARKET" => {
            if let Some(qty) = quantity {
                params.push(format!("quantity={}", qty));
            } else {
                return Err(ProviderError::Api {
                    status: None,
                    message: "Quantité requise pour les ordres MARKET".to_string(),
                });
            }
        }
        "LIMIT" => {
            if let Some(qty) = quantity {
                params.push(format!("quantity={}", qty));
            } else {
                return Err(ProviderError::Api {
                    status: None,
                    message: "Quantité requise pour les ordres LIMIT".to_string(),
                });
            }

            if let Some(p) = price {
                params.push(format!("price={}", p));
            } else {
                return Err(ProviderError::Api {
                    status: None,
                    message: "Prix requis pour les ordres LIMIT".to_string(),
                });
            }

            // Time in force (GTC par défaut)
            let tif = time_in_force.unwrap_or("GTC");
            params.push(format!("timeInForce={}", tif));
        }
        _ => {
            return Err(ProviderError::Api {
                status: None,
                message: format!("Type d'ordre non supporté: {}", order_type),
            });
        }
    }

    // Construire la query string
    let query_string = params.join("&");

    // Générer la signature HMAC
    let signature = provider.generate_signature(&query_string)?;
    let query_string_with_sig = format!("{}&signature={}", query_string, signature);

    // Construire l'URL
    let url = format!("{}/order?{}", provider.base_url, query_string_with_sig);

    // Faire la requête POST
    let response = provider
        .client
        .post(&url)
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let status = response.status();
    let status_code = status.as_u16();

    if status.is_success() {
        let order_response: OrderResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(format!("Erreur parsing JSON: {}", e)))?;

        Ok(order_response)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Erreur inconnue".to_string());

        // Parser les erreurs Binance communes
        let error_message = parse_binance_error(&error_text, status_code);

        Err(ProviderError::Api {
            status: Some(status_code),
            message: error_message,
        })
    }
}

/// Parse les erreurs Binance communes et retourne un message utilisateur-friendly
fn parse_binance_error(error_text: &str, status_code: u16) -> String {
    // Codes d'erreur Binance courants
    if error_text.contains("-2010") || error_text.contains("NEW_ORDER_REJECTED") {
        return format!("Ordre rejeté: {}", error_text);
    }
    if error_text.contains("-2011") || error_text.contains("UNKNOWN_ORDER") {
        return format!("Ordre inconnu: {}", error_text);
    }
    if error_text.contains("-2013") || error_text.contains("NO_SUCH_SYMBOL") {
        return format!("Symbole non trouvé: {}", error_text);
    }
    if error_text.contains("-1013") || error_text.contains("INVALID_PRICE") {
        return format!("Prix invalide: {}", error_text);
    }
    if error_text.contains("-1010") || error_text.contains("INVALID_QUANTITY") {
        return format!("Quantité invalide: {}", error_text);
    }
    if error_text.contains("-2019") || error_text.contains("INSUFFICIENT_BALANCE") {
        return format!("Solde insuffisant: {}", error_text);
    }
    if error_text.contains("-1021") || error_text.contains("TIMESTAMP") {
        return format!("Erreur de synchronisation temporelle: {}", error_text);
    }
    if error_text.contains("-1022") || error_text.contains("INVALID_SIGNATURE") {
        return format!("Signature invalide: {}", error_text);
    }
    if error_text.contains("-2015") || error_text.contains("INVALID_API_KEY") {
        return format!("Clé API invalide: {}", error_text);
    }
    if status_code == 429 {
        return format!("Rate limit dépassé. Veuillez patienter avant de réessayer.");
    }
    if status_code == 418 {
        return format!("IP bannie temporairement pour cause de rate limit excessif.");
    }

    // Erreur générique
    format!("Erreur API Binance ({}): {}", status_code, error_text)
}

/// Récupère le statut d'un ordre
pub async fn get_order_status(
    provider: &BinanceProvider,
    symbol: &str,
    order_id: u64,
) -> Result<OrderResponse, ProviderError> {
    // Vérifier que le provider a les credentials nécessaires
    if provider.api_token.is_none() || provider.api_secret.is_none() {
        return Err(ProviderError::Api {
            status: None,
            message: "Configuration API manquante (token et secret requis)".to_string(),
        });
    }

    // Générer le timestamp en millisecondes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ProviderError::Api {
            status: None,
            message: format!("Erreur génération timestamp: {}", e),
        })?
        .as_millis() as u64;

    // Construire la query string
    let query_string = format!(
        "symbol={}&orderId={}&timestamp={}",
        symbol.to_uppercase(),
        order_id,
        timestamp
    );

    // Générer la signature HMAC
    let signature = provider.generate_signature(&query_string)?;
    let query_string_with_sig = format!("{}&signature={}", query_string, signature);

    // Construire l'URL
    let url = format!("{}/order?{}", provider.base_url, query_string_with_sig);

    // Faire la requête GET
    let response = provider
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let status = response.status();
    let status_code = status.as_u16();

    if status.is_success() {
        let order_response: OrderResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(format!("Erreur parsing JSON: {}", e)))?;

        Ok(order_response)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Erreur inconnue".to_string());

        let error_message = parse_binance_error(&error_text, status_code);

        Err(ProviderError::Api {
            status: Some(status_code),
            message: error_message,
        })
    }
}

/// Annule un ordre
pub async fn cancel_order(
    provider: &BinanceProvider,
    symbol: &str,
    order_id: u64,
) -> Result<OrderResponse, ProviderError> {
    // Vérifier que le provider a les credentials nécessaires
    if provider.api_token.is_none() || provider.api_secret.is_none() {
        return Err(ProviderError::Api {
            status: None,
            message: "Configuration API manquante (token et secret requis)".to_string(),
        });
    }

    // Générer le timestamp en millisecondes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ProviderError::Api {
            status: None,
            message: format!("Erreur génération timestamp: {}", e),
        })?
        .as_millis() as u64;

    // Construire la query string
    let query_string = format!(
        "symbol={}&orderId={}&timestamp={}",
        symbol.to_uppercase(),
        order_id,
        timestamp
    );

    // Générer la signature HMAC
    let signature = provider.generate_signature(&query_string)?;
    let query_string_with_sig = format!("{}&signature={}", query_string, signature);

    // Construire l'URL
    let url = format!("{}/order?{}", provider.base_url, query_string_with_sig);

    // Faire la requête DELETE
    let response = provider
        .client
        .delete(&url)
        .send()
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

    let status = response.status();
    let status_code = status.as_u16();

    if status.is_success() {
        let order_response: OrderResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Parse(format!("Erreur parsing JSON: {}", e)))?;

        Ok(order_response)
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Erreur inconnue".to_string());

        let error_message = parse_binance_error(&error_text, status_code);

        Err(ProviderError::Api {
            status: Some(status_code),
            message: error_message,
        })
    }
}
