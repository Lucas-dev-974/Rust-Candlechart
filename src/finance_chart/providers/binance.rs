//! Provider Binance pour la mise à jour en temps réel
//!
//! Implémente `RealtimeDataProvider` pour récupérer les données depuis l'API Binance.

use crate::finance_chart::core::{Candle, SeriesId};
use crate::finance_chart::realtime::{RealtimeDataProvider, ProviderError};
use std::time::Duration;

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
}

impl BinanceProvider {
    /// Crée un nouveau provider Binance avec les paramètres par défaut
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Crée un nouveau provider avec un timeout personnalisé
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_config(timeout, None)
    }

    /// Crée un nouveau provider avec un token API
    pub fn with_token(api_token: Option<String>) -> Self {
        Self::with_config(Duration::from_secs(DEFAULT_TIMEOUT_SECS), api_token)
    }

    /// Crée un nouveau provider avec une configuration complète
    pub fn with_config(timeout: Duration, api_token: Option<String>) -> Self {
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
        }
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

    /// Récupère toutes les bougies de manière asynchrone
    pub async fn fetch_all_candles_async(&self, series_id: &SeriesId) -> Result<Vec<Candle>, ProviderError> {
        self.fetch_new_candles_async(series_id, 0).await
    }

    /// Récupère les bougies dans une plage temporelle spécifique
    pub async fn fetch_candles_in_range_async(
        &self,
        series_id: &SeriesId,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let start_time_ms = start_timestamp * 1000;
        let end_time_ms = end_timestamp * 1000;
        self.fetch_klines(&symbol, &interval, Some(start_time_ms), Some(end_time_ms), Some(1000)).await
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
        let interval = parts[1..].join("_").to_lowercase();

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

    /// Teste la connexion avec authentification
    pub async fn test_authenticated_connection(&self) -> Result<(), ProviderError> {
        if self.api_token.is_none() {
            return Err(ProviderError::Api {
                status: None,
                message: "Aucun token API configuré".to_string(),
            });
        }

        let url = format!("{}/account", self.base_url);
        
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

