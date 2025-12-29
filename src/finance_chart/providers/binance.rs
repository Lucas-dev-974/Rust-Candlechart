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

