//! Module API pour le trading réel via Binance
//!
//! Ce module gère les appels API pour placer des ordres réels sur Binance,
//! avec validation, gestion d'erreurs et suivi des ordres.

use crate::finance_chart::providers::binance::BinanceProvider;
use crate::finance_chart::realtime::ProviderError;
use crate::app::error_handling::{retry_with_backoff, RetryConfig, AppError};
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

/// Place un ordre Market BUY sur Binance avec retry automatique
pub async fn place_market_buy_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
) -> Result<OrderResponse, ProviderError> {
    let context = format!("placement d'ordre Market BUY {}", symbol);
    place_order_with_retry(
        provider,
        symbol,
        "BUY",
        "MARKET",
        Some(quantity),
        None,
        None,
        &context,
    )
    .await
}

/// Place un ordre Market SELL sur Binance avec retry automatique
pub async fn place_market_sell_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
) -> Result<OrderResponse, ProviderError> {
    let context = format!("placement d'ordre Market SELL {}", symbol);
    place_order_with_retry(
        provider,
        symbol,
        "SELL",
        "MARKET",
        Some(quantity),
        None,
        None,
        &context,
    )
    .await
}

/// Place un ordre Limit BUY sur Binance avec retry automatique
pub async fn place_limit_buy_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
    price: f64,
    time_in_force: Option<&str>,
) -> Result<OrderResponse, ProviderError> {
    let context = format!("placement d'ordre Limit BUY {}", symbol);
    place_order_with_retry(
        provider,
        symbol,
        "BUY",
        "LIMIT",
        Some(quantity),
        Some(price),
        time_in_force,
        &context,
    )
    .await
}

/// Place un ordre Limit SELL sur Binance avec retry automatique
pub async fn place_limit_sell_order(
    provider: &BinanceProvider,
    symbol: &str,
    quantity: f64,
    price: f64,
    time_in_force: Option<&str>,
) -> Result<OrderResponse, ProviderError> {
    let context = format!("placement d'ordre Limit SELL {}", symbol);
    place_order_with_retry(
        provider,
        symbol,
        "SELL",
        "LIMIT",
        Some(quantity),
        Some(price),
        time_in_force,
        &context,
    )
    .await
}

/// Fonction helper pour placer un ordre avec retry automatique
async fn place_order_with_retry(
    provider: &BinanceProvider,
    symbol: &str,
    side: &str,
    order_type: &str,
    quantity: Option<f64>,
    price: Option<f64>,
    time_in_force: Option<&str>,
    context: &str,
) -> Result<OrderResponse, ProviderError> {
    let provider = provider.clone();
    let symbol = symbol.to_string();
    let side = side.to_string();
    let order_type = order_type.to_string();
    let time_in_force = time_in_force.map(|s| s.to_string());
    let context = context.to_string();
    
    let context_for_log = context.clone();
    let retry_result = retry_with_backoff(
        {
            let provider = provider.clone();
            let symbol = symbol.clone();
            let side = side.clone();
            let order_type = order_type.clone();
            let time_in_force = time_in_force.clone();
            let context = context.clone();
            move || {
                let provider = provider.clone();
                let symbol = symbol.clone();
                let side = side.clone();
                let order_type = order_type.clone();
                let time_in_force_clone = time_in_force.clone();
                let quantity = quantity;
                let price = price;
                let context = context.clone();
                async move {
                    place_order_internal(
                        &provider,
                        &symbol,
                        &side,
                        &order_type,
                        quantity,
                        price,
                        time_in_force_clone.as_deref(),
                    )
                    .await
                    .map_err(|e| {
                        let ctx = context.clone();
                        AppError::from_provider_error(e, &ctx)
                    })
                }
            }
        },
        RetryConfig::for_api_calls(),
        &context_for_log,
    )
    .await;
    
    match retry_result {
        crate::app::error_handling::RetryResult::Success { value, .. } => Ok(value),
        crate::app::error_handling::RetryResult::Failed { error, .. } => {
            Err(ProviderError::Api {
                status: None,
                message: error.user_message,
            })
        }
    }
}

/// Fonction générique pour placer un ordre sur Binance (utilisée par les fonctions publiques avec retry)
async fn place_order_internal(
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
    // Essayer de parser le JSON d'erreur Binance si disponible
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(error_text) {
        if let Some(code) = json_value.get("code").and_then(|v| v.as_i64()) {
            let msg = json_value.get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            match code {
                -2010 => return format!("Ordre rejeté: {}", msg),
                -2011 => return format!("Ordre inconnu: {}", msg),
                -2013 => return format!("Symbole non trouvé: {}", msg),
                -1013 => return format!("Prix invalide: {}", msg),
                -1010 => return format!("Quantité invalide: {}", msg),
                -2019 => return format!("Solde insuffisant: {}", msg),
                -1021 => return format!("Erreur de synchronisation temporelle. Vérifiez l'heure de votre système: {}", msg),
                -1022 => return format!("Signature invalide. Vérifiez votre clé secrète API: {}", msg),
                -2015 => return format!("Clé API invalide ou expirée: {}", msg),
                -2014 => return format!("Clé API manquante: {}", msg),
                -1102 => return format!("Paramètre manquant: {}", msg),
                -1104 => return format!("Paramètre obligatoire manquant: {}", msg),
                -1105 => return format!("Paramètre invalide: {}", msg),
                -1106 => return format!("Paramètre invalide (type): {}", msg),
                -1111 => return format!("Précision invalide: {}", msg),
                -1112 => return format!("Nombre invalide: {}", msg),
                -1114 => return format!("Prix de marché non disponible: {}", msg),
                -1115 => return format!("Prix invalide (trop bas ou trop haut): {}", msg),
                -1116 => return format!("Quantité invalide (trop basse ou trop haute): {}", msg),
                -1117 => return format!("Prix invalide (trop de décimales): {}", msg),
                -1118 => return format!("Quantité invalide (trop de décimales): {}", msg),
                -1119 => return format!("Multiplicateur invalide: {}", msg),
                -1120 => return format!("Quantité invalide (doit être un multiple de): {}", msg),
                -1121 => return format!("Symbole invalide: {}", msg),
                -1125 => return format!("Filtre invalide: {}", msg),
                -1127 => return format!("Listen key invalide: {}", msg),
                -1128 => return format!("Plus de 24 heures depuis la création de la listen key: {}", msg),
                -2021 => return format!("Ordre rejeté (trop de requêtes): {}", msg),
                _ => return format!("Erreur Binance (code {}): {}", code, msg),
            }
        }
    }
    
    // Fallback: parsing par texte
    let error_lower = error_text.to_lowercase();
    
    if error_lower.contains("-2010") || error_lower.contains("new_order_rejected") {
        return format!("Ordre rejeté. Vérifiez les paramètres de votre ordre.");
    }
    if error_lower.contains("-2011") || error_lower.contains("unknown_order") {
        return format!("Ordre inconnu. L'ordre spécifié n'existe pas.");
    }
    if error_lower.contains("-2013") || error_lower.contains("no_such_symbol") {
        return format!("Symbole non trouvé. Vérifiez que le symbole existe et est tradable.");
    }
    if error_lower.contains("-1013") || error_lower.contains("invalid_price") {
        return format!("Prix invalide. Le prix doit respecter les règles de précision du symbole.");
    }
    if error_lower.contains("-1010") || error_lower.contains("invalid_quantity") {
        return format!("Quantité invalide. La quantité doit respecter les règles de précision du symbole.");
    }
    if error_lower.contains("-2019") || error_lower.contains("insufficient_balance") {
        return format!("Solde insuffisant. Vous n'avez pas assez de fonds pour cet ordre.");
    }
    if error_lower.contains("-1021") || error_lower.contains("timestamp") {
        return format!("Erreur de synchronisation temporelle. Vérifiez que l'heure de votre système est correcte.");
    }
    if error_lower.contains("-1022") || error_lower.contains("invalid_signature") {
        return format!("Signature invalide. Vérifiez votre clé secrète API.");
    }
    if error_lower.contains("-2015") || error_lower.contains("invalid_api_key") {
        return format!("Clé API invalide ou expirée. Vérifiez vos identifiants API.");
    }
    if error_lower.contains("-2014") || error_lower.contains("missing_api_key") {
        return format!("Clé API manquante. Configurez vos identifiants API.");
    }
    if status_code == 429 {
        return format!("Trop de requêtes. Veuillez patienter quelques instants avant de réessayer.");
    }
    if status_code == 418 {
        return format!("IP bannie temporairement pour cause de rate limit excessif. Attendez avant de réessayer.");
    }
    if status_code == 401 {
        return format!("Non autorisé. Vérifiez vos clés API.");
    }
    if status_code == 403 {
        return format!("Accès interdit. Vérifiez les permissions de votre clé API.");
    }
    if status_code >= 500 {
        return format!("Erreur serveur Binance. Réessayez dans quelques instants.");
    }

    // Erreur générique avec extraction du message si possible
    let clean_msg = if error_text.len() > 200 {
        format!("{}...", &error_text[..200])
    } else {
        error_text.to_string()
    };
    
    format!("Erreur API Binance ({}): {}", status_code, clean_msg)
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
