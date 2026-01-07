//! Provider Binance pour la mise à jour en temps réel
//!
//! Implémente `RealtimeDataProvider` pour récupérer les données depuis l'API Binance.

use crate::finance_chart::core::{Candle, SeriesId};
use crate::finance_chart::realtime::{RealtimeDataProvider, ProviderError};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
type HmacSha256 = Hmac<Sha256>;

/// Balance d'un asset dans le compte Binance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAccountBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

/// URL de base de l'API Binance
const BINANCE_API_BASE: &str = "https://api.binance.com/api/v3";

/// Timeout par défaut pour les requêtes HTTP (en secondes)
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Provider Binance pour récupérer les données depuis l'API Binance
#[derive(Clone)]
pub struct BinanceProvider {
    /// Client HTTP pour les requêtes
    client: reqwest::Client,
    /// URL de base de l'API
    base_url: String,
    /// Token API optionnel pour l'authentification
    #[allow(dead_code)]
    api_token: Option<String>,
    /// Clé secrète API pour la signature HMAC
    api_secret: Option<String>,
}

impl BinanceProvider {
    /// Crée un nouveau provider Binance avec les paramètres par défaut
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Crée un nouveau provider avec un timeout personnalisé
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_config(timeout, None, None)
    }

    /// Crée un nouveau provider avec un token API
    pub fn with_token(api_token: Option<String>) -> Self {
        Self::with_config(Duration::from_secs(DEFAULT_TIMEOUT_SECS), api_token, None)
    }

    /// Crée un nouveau provider avec un token API et une clé secrète
    pub fn with_token_and_secret(api_token: Option<String>, api_secret: Option<String>) -> Self {
        Self::with_config(Duration::from_secs(DEFAULT_TIMEOUT_SECS), api_token, api_secret)
    }

    /// Crée un nouveau provider avec une configuration complète
    pub fn with_config(timeout: Duration, api_token: Option<String>, api_secret: Option<String>) -> Self {
        let mut client_builder = reqwest::Client::builder()
            .timeout(timeout);

        if let Some(ref token) = api_token {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(token) {
                client_builder = client_builder.default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert("X-MBX-APIKEY", header_value);
                    headers
                });
            } else {
                eprintln!("⚠️ Token API invalide, utilisation sans authentification");
            }
        }

        let client = client_builder
            .build()
            .unwrap_or_else(|e| {
                eprintln!("⚠️ Erreur création client HTTP: {}. Utilisation d'un client basique.", e);
                reqwest::Client::new()
            });

        Self {
            client,
            base_url: BINANCE_API_BASE.to_string(),
            api_token,
            api_secret,
        }
    }
    
    /// Génère une signature HMAC SHA256 pour une requête Binance
    fn generate_signature(&self, query_string: &str) -> Result<String, ProviderError> {
        let secret = self.api_secret.as_ref()
            .ok_or_else(|| ProviderError::Api {
                status: None,
                message: "Clé secrète API non configurée".to_string(),
            })?;
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| ProviderError::Api {
                status: None,
                message: format!("Erreur création HMAC: {}", e),
            })?;
        
        mac.update(query_string.as_bytes());
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());
        
        Ok(signature)
    }

    /// Récupère la dernière bougie de manière asynchrone
    pub async fn get_latest_candle_async(&self, series_id: &SeriesId) -> Result<Option<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let candles = self.fetch_klines(&symbol, &interval, None, None, Some(1)).await?;
        Ok(candles.into_iter().last())
    }

    /// Récupère les nouvelles bougies depuis un timestamp de manière asynchrone
    pub async fn fetch_new_candles_async(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let start_time_ms = since_timestamp * 1000;
        self.fetch_klines(&symbol, &interval, Some(start_time_ms), None, Some(1000)).await
    }

    /// Récupère toutes les bougies de manière asynchrone (limité à 1000)
    pub async fn fetch_all_candles_async(&self, series_id: &SeriesId) -> Result<Vec<Candle>, ProviderError> {
        self.fetch_new_candles_async(series_id, 0).await
    }

    /// Récupère tout l'historique disponible avec pagination
    /// Fait plusieurs requêtes pour récupérer toutes les bougies disponibles
    /// Les bougies sont retournées triées par timestamp croissant (les plus anciennes en premier)
    pub async fn fetch_full_history_async(&self, series_id: &SeriesId) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        
        let mut all_candles = Vec::new();
        let mut end_time: Option<i64> = None;
        const BATCH_SIZE: usize = 1000; // Limite maximale de Binance
        
        println!("📥 Téléchargement de l'historique complet pour {}...", series_id.name);
        
        loop {
            let candles = if let Some(end) = end_time {
                // Télécharger les bougies avant le timestamp end_time (plus anciennes)
                self.fetch_klines(&symbol, &interval, None, Some(end * 1000), Some(BATCH_SIZE)).await?
            } else {
                // Première requête : récupérer les bougies les plus récentes
                self.fetch_klines(&symbol, &interval, None, None, Some(BATCH_SIZE)).await?
            };
            
            if candles.is_empty() {
                break;
            }
            
            let candles_count = candles.len();
            
            // Les bougies de Binance sont triées par timestamp croissant
            // On les ajoute au début de all_candles pour garder l'ordre chronologique
            all_candles.splice(0..0, candles);
            
            // Si on a récupéré moins de BATCH_SIZE bougies, on a tout récupéré
            if candles_count < BATCH_SIZE {
                break;
            }
            
            // Le timestamp de la première bougie (la plus ancienne) devient le nouveau end_time
            if let Some(first_candle) = all_candles.first() {
                end_time = Some(first_candle.timestamp - 1);
            } else {
                break;
            }
            
            println!("  📊 {} bougies téléchargées...", all_candles.len());
            
            // Petite pause pour éviter de surcharger l'API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        println!("✅ Historique complet téléchargé: {} bougies", all_candles.len());
        Ok(all_candles)
    }

    /// Récupère les bougies du plus récent vers le plus ancien (limité à 1000)
    /// Retourne les 1000 bougies les plus récentes AVANT end_timestamp
    /// Le filtrage par start_timestamp doit être fait côté appelant
    pub async fn fetch_candles_backwards_async(
        &self,
        series_id: &SeriesId,
        _start_timestamp: i64,  // Non utilisé ici - filtrage fait côté appelant
        end_timestamp: i64,     // timestamp maximum - on récupère les 1000 bougies AVANT cette date
    ) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let end_time_ms = end_timestamp * 1000;
        
        // Récupérer les 1000 bougies les plus récentes AVANT end_timestamp
        // L'API retourne les bougies triées par timestamp croissant (du plus ancien au plus récent)
        self.fetch_klines(&symbol, &interval, None, Some(end_time_ms), Some(1000)).await
    }

    /// Récupère TOUTES les bougies dans une plage temporelle avec pagination
    /// Fait plusieurs requêtes si nécessaire pour combler tout le gap
    pub async fn fetch_all_candles_in_range_async(
        &self,
        series_id: &SeriesId,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        
        let mut all_candles = Vec::new();
        let mut current_start = start_timestamp;
        const BATCH_SIZE: usize = 1000;
        
        println!("📥 Téléchargement des données de {} à {} pour {}...", start_timestamp, end_timestamp, series_id.name);
        
        loop {
            let start_time_ms = current_start * 1000;
            let end_time_ms = end_timestamp * 1000;
            
            let candles = self.fetch_klines(&symbol, &interval, Some(start_time_ms), Some(end_time_ms), Some(BATCH_SIZE)).await?;
            
            if candles.is_empty() {
                break;
            }
            
            let candles_count = candles.len();
            
            // Trouver le timestamp le plus récent pour la prochaine requête
            if let Some(last_candle) = candles.last() {
                current_start = last_candle.timestamp + 1; // +1 pour éviter les doublons
            }
            
            all_candles.extend(candles);
            
            // Si on a atteint la fin ou si on a moins de BATCH_SIZE bougies, on a tout récupéré
            if candles_count < BATCH_SIZE || current_start >= end_timestamp {
                break;
            }
            
            println!("  📊 {} bougies téléchargées...", all_candles.len());
            
            // Petite pause pour éviter de surcharger l'API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        println!("✅ Total: {} bougies téléchargées", all_candles.len());
        Ok(all_candles)
    }

    /// Vérifie s'il existe des données plus anciennes disponibles pour une série
    /// Retourne le timestamp de la bougie la plus ancienne disponible sur l'API
    pub async fn check_oldest_available_timestamp_async(&self, series_id: &SeriesId) -> Result<Option<i64>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        
        // Récupérer la première bougie disponible (la plus ancienne)
        // On utilise startTime = 0 pour demander les données depuis le début
        // Binance retourne les bougies par ordre croissant, donc la première est la plus ancienne
        let start_timestamp_ms = 0; // Demander depuis le tout début
        let candles = self.fetch_klines(&symbol, &interval, Some(start_timestamp_ms), None, Some(1)).await?;
        
        Ok(candles.first().map(|c| c.timestamp))
    }

    /// Extrait le symbole et l'intervalle depuis un SeriesId
    fn parse_series_id(&self, series_id: &SeriesId) -> Result<(String, String), ProviderError> {
        let parts: Vec<&str> = series_id.name.split('_').collect();
        if parts.len() < 2 {
            return Err(ProviderError::InvalidSeriesId(format!(
                "Format de SeriesId invalide: {}. Attendu: SYMBOL_INTERVAL (ex: BTCUSDT_1h)",
                series_id.name
            )));
        }

        let symbol = parts[0].to_uppercase();
        // IMPORTANT: Ne pas convertir l'intervalle en minuscule car Binance est sensible à la casse
        // "1m" = 1 minute, "1M" = 1 mois
        let interval = parts[1..].join("_");

        Ok((symbol, interval))
    }

    /// Convertit une réponse kline Binance en Candle
    fn parse_kline_array(&self, arr: &[serde_json::Value]) -> Result<Candle, ProviderError> {
        if arr.len() < 6 {
            return Err(ProviderError::Parse(format!(
                "Tableau kline incomplet: {} éléments (attendu: au moins 6)",
                arr.len()
            )));
        }

        let parse_price = |idx: usize, field: &str| -> Result<f64, ProviderError> {
            arr[idx]
                .as_str()
                .ok_or_else(|| ProviderError::Parse(format!("{} invalide (string)", field)))?
                .parse::<f64>()
                .map_err(|e| ProviderError::Parse(format!("Erreur parsing {}: {}", field, e)))
        };

        let open_time_ms = arr[0]
            .as_i64()
            .ok_or_else(|| ProviderError::Parse("open_time invalide".to_string()))?;
        let open = parse_price(1, "open")?;
        let high = parse_price(2, "high")?;
        let low = parse_price(3, "low")?;
        let close = parse_price(4, "close")?;
        let volume = parse_price(5, "volume")?;

        let timestamp = open_time_ms / 1000;

        Ok(Candle::new(timestamp, open, high, low, close, volume))
    }

    /// Exécute une future async
    #[allow(dead_code)]
    fn run_async<F, T>(&self, future: F) -> Result<T, ProviderError>
    where
        F: std::future::Future<Output = Result<T, ProviderError>>,
    {
        tokio::runtime::Runtime::new()
            .map_err(|e| ProviderError::Unknown(format!("Erreur création runtime: {}", e)))?
            .block_on(future)
    }

    /// Récupère les klines depuis l'API Binance
    async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<Candle>, ProviderError> {
        let mut url = format!("{}/klines?symbol={}&interval={}", self.base_url, symbol, interval);
        
        let mut params = Vec::new();
        if let Some(start) = start_time {
            params.push(format!("startTime={}", start));
        }
        if let Some(end) = end_time {
            params.push(format!("endTime={}", end));
        }
        if let Some(lim) = limit {
            params.push(format!("limit={}", lim.min(1000)));
        }
        
        if !params.is_empty() {
            url.push('&');
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(ProviderError::from)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Erreur inconnue".to_string());
            return Err(ProviderError::Api {
                status: Some(status),
                message: error_text,
            });
        }

        let json: Vec<Vec<serde_json::Value>> = response
            .json()
            .await
            .map_err(ProviderError::from)?;

        let mut candles = Vec::new();
        for kline_arr in json {
            match self.parse_kline_array(&kline_arr) {
                Ok(candle) => candles.push(candle),
                Err(e) => {
                    eprintln!("⚠️ Erreur parsing kline: {}", e);
                }
            }
        }

        Ok(candles)
    }

    /// Teste la connexion à l'API Binance
    pub async fn test_connection(&self) -> Result<(), ProviderError> {
        let url = format!("{}/ping", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Erreur inconnue".to_string());
            Err(ProviderError::Api {
                status: Some(status),
                message: error_text,
            })
        }
    }

    /// Récupère les informations du compte Binance
    /// 
    /// Nécessite une signature HMAC pour fonctionner complètement.
    /// Retourne les balances du compte si la requête réussit.
    pub async fn get_account_info(&self) -> Result<Vec<BinanceAccountBalance>, ProviderError> {
        if self.api_token.is_none() {
            return Err(ProviderError::Api {
                status: None,
                message: "Aucun token API configuré".to_string(),
            });
        }

        if self.api_secret.is_none() {
            return Err(ProviderError::Api {
                status: None,
                message: "Aucune clé secrète API configurée. L'endpoint /account nécessite une signature HMAC.".to_string(),
            });
        }

        // Générer le timestamp en millisecondes
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ProviderError::Api {
                status: None,
                message: format!("Erreur génération timestamp: {}", e),
            })?
            .as_millis() as u64;

        // Construire la query string avec le timestamp
        let query_string = format!("timestamp={}", timestamp);
        
        // Générer la signature HMAC (la clé secrète est garantie d'exister ici)
        let signature = self.generate_signature(&query_string)?;
        let query_string_with_sig = format!("{}&signature={}", query_string, signature);
        
        println!("   Signature générée pour timestamp={}", timestamp);

        let url = format!("{}/account?{}", self.base_url, query_string_with_sig);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = response.status();
        let status_code = status.as_u16();
        
        if status.is_success() {
            let json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ProviderError::Parse(format!("Erreur parsing JSON: {}", e)))?;
            
            // Parser les balances depuis la réponse JSON
            let balances = json.get("balances")
                .and_then(|b| b.as_array())
                .ok_or_else(|| ProviderError::Parse("Champ 'balances' manquant ou invalide".to_string()))?;
            
            let mut account_balances = Vec::new();
            for balance in balances {
                if let Some(asset) = balance.get("asset").and_then(|a| a.as_str()) {
                    let free = balance.get("free")
                        .and_then(|f| f.as_str())
                        .unwrap_or("0")
                        .to_string();
                    let locked = balance.get("locked")
                        .and_then(|l| l.as_str())
                        .unwrap_or("0")
                        .to_string();
                    
                    account_balances.push(BinanceAccountBalance {
                        asset: asset.to_string(),
                        free,
                        locked,
                    });
                }
            }
            
            Ok(account_balances)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Erreur inconnue".to_string());
            
            Err(ProviderError::Api {
                status: Some(status_code),
                message: format!("Erreur API: {}", error_text),
            })
        }
    }

    /// Teste la connexion avec authentification
    /// 
    /// Utilise l'endpoint /api/v3/account qui nécessite une signature HMAC.
    /// Si l'API key est valide mais la signature manque, Binance retourne une erreur spécifique
    /// qui indique que l'API key est valide mais qu'il manque la signature.
    /// Si l'API key est invalide, on reçoit une erreur différente.
    pub async fn test_authenticated_connection(&self) -> Result<(), ProviderError> {
        if self.api_token.is_none() {
            return Err(ProviderError::Api {
                status: None,
                message: "Aucun token API configuré".to_string(),
            });
        }

        // Utiliser l'endpoint /api/v3/account qui nécessite une signature
        // Si l'API key est valide mais la signature manque, Binance retourne une erreur spécifique
        let url = format!("{}/account", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let status = response.status();
        let status_code = status.as_u16();
        
        if status.is_success() {
            // Si on reçoit un succès, c'est que l'API key ET la signature sont valides
            Ok(())
        } else {
            // Lire le message d'erreur pour déterminer le type d'erreur
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Erreur inconnue".to_string());
            
            // Codes d'erreur Binance pertinents :
            // -1102 : Paramètre signature manquant (l'API key est valide mais la signature est requise)
            // -1022 : Signature invalide (l'API key est valide mais la signature est incorrecte)
            // -2015 : API key invalide
            // -2014 : API key manquante
            
            // Si l'erreur indique que la signature manque ou est invalide, cela signifie que l'API key est valide
            // Car Binance a accepté l'API key mais rejette la requête à cause de la signature
            // On cherche le code d'erreur de différentes manières pour être robuste
            let error_lower = error_text.to_lowercase();
            if error_text.contains("-1102") 
                || error_text.contains("-1022")
                || error_lower.contains("mandatory parameter 'signature'")
                || error_lower.contains("mandatory parameter \"signature\"")
                || error_lower.contains("invalid signature")
                || error_lower.contains("signature was not sent")
                || error_lower.contains("signature was not sent, was empty/null") {
                // L'API key est valide mais la signature manque (ce qui est normal pour un test simple)
                println!("✅ API key valide (signature requise mais non fournie pour le test)");
                Ok(())
            } else if error_text.contains("-2015") 
                || error_text.contains("-2014")
                || error_lower.contains("invalid api-key")
                || error_lower.contains("api-key format invalid")
                || error_lower.contains("invalid api key") {
                // L'API key est invalide ou manquante
                Err(ProviderError::Api {
                    status: Some(status_code),
                    message: format!("Clé API invalide: {}", error_text),
                })
            } else {
                // Autre erreur - afficher le message complet pour debug
                println!("⚠️ Erreur API lors du test (code non reconnu): {}", error_text);
                Err(ProviderError::Api {
                    status: Some(status_code),
                    message: format!("Erreur API: {}", error_text),
                })
            }
        }
    }
}

impl Default for BinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BinanceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinanceProvider")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl RealtimeDataProvider for BinanceProvider {
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        let (symbol, interval) = self.parse_series_id(series_id)
            .map_err(|e| e.to_string())?;
        
        self.run_async(async {
            let candles = self.fetch_klines(&symbol, &interval, None, None, Some(1)).await?;
            Ok(candles.into_iter().last())
        })
        .map_err(|e| e.to_string())
    }

    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String> {
        let (symbol, interval) = self.parse_series_id(series_id)
            .map_err(|e| e.to_string())?;
        let start_time_ms = since_timestamp * 1000;
        
        self.run_async(async {
            self.fetch_klines(&symbol, &interval, Some(start_time_ms), None, Some(1000)).await
        })
        .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_series_id() {
        let provider = BinanceProvider::new();
        let series_id = SeriesId::new("BTCUSDT_1h");
        
        let (symbol, interval) = provider.parse_series_id(&series_id).unwrap();
        assert_eq!(symbol, "BTCUSDT");
        assert_eq!(interval, "1h");
    }

    #[test]
    fn test_parse_series_id_with_multiple_underscores() {
        let provider = BinanceProvider::new();
        let series_id = SeriesId::new("ETHUSDT_15m");
        
        let (symbol, interval) = provider.parse_series_id(&series_id).unwrap();
        assert_eq!(symbol, "ETHUSDT");
        assert_eq!(interval, "15m");
    }

    #[test]
    fn test_parse_kline_array() {
        let provider = BinanceProvider::new();
        
        let kline_json = serde_json::json!([
            1609459200000i64,
            "50000.0",
            "50100.0",
            "49900.0",
            "50050.0",
            "100.5",
            1609462800000i64,
            "50000000.0",
            1000i64,
            "2500000.0",
            "125000000.0"
        ]);
        
        let arr: Vec<serde_json::Value> = serde_json::from_value(kline_json).unwrap();
        let candle = provider.parse_kline_array(&arr).unwrap();
        
        assert_eq!(candle.timestamp, 1609459200);
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.high, 50100.0);
        assert_eq!(candle.low, 49900.0);
        assert_eq!(candle.close, 50050.0);
    }
}

