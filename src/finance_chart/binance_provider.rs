//! Provider Binance pour la mise à jour en temps réel
//!
//! Implémente `RealtimeDataProvider` pour récupérer les données depuis l'API Binance.
//!
//! # Exemple d'utilisation
//!
//! ```ignore
//! use candlechart::BinanceProvider;
//!
//! let provider = BinanceProvider::new();
//! let result = chart_state.update_from_provider(&series_id, &provider);
//! ```

use super::core::{Candle, SeriesId};
use super::realtime::{RealtimeDataProvider, ProviderError};
use std::time::Duration;

/// URL de base de l'API Binance
const BINANCE_API_BASE: &str = "https://api.binance.com/api/v3";

/// Timeout par défaut pour les requêtes HTTP (en secondes)
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Provider Binance pour récupérer les données depuis l'API Binance
///
/// Utilise l'API REST publique de Binance pour récupérer les données de klines (bougies).
///
/// # Exemple
///
/// ```ignore
/// let provider = BinanceProvider::new();
/// let provider = BinanceProvider::with_timeout(Duration::from_secs(5));
/// ```
#[derive(Clone)]
pub struct BinanceProvider {
    /// Client HTTP pour les requêtes
    client: reqwest::Client,
    /// URL de base de l'API (par défaut: API publique Binance)
    base_url: String,
    /// Token API optionnel pour l'authentification
    #[allow(dead_code)] // Stocké pour usage futur (authentification API)
    api_token: Option<String>,
}

impl BinanceProvider {
    /// Crée un nouveau provider Binance avec les paramètres par défaut
    ///
    /// Utilise l'API publique de Binance avec un timeout de 10 secondes.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    }

    /// Crée un nouveau provider avec un timeout personnalisé
    ///
    /// # Arguments
    /// * `timeout` - Timeout pour les requêtes HTTP
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::with_config(timeout, None)
    }

    /// Crée un nouveau provider avec un token API
    ///
    /// # Arguments
    /// * `api_token` - Token API pour l'authentification (optionnel)
    pub fn with_token(api_token: Option<String>) -> Self {
        Self::with_config(Duration::from_secs(DEFAULT_TIMEOUT_SECS), api_token)
    }

    /// Crée un nouveau provider avec une configuration complète
    ///
    /// # Arguments
    /// * `timeout` - Timeout pour les requêtes HTTP
    /// * `api_token` - Token API pour l'authentification (optionnel)
    pub fn with_config(timeout: Duration, api_token: Option<String>) -> Self {
        let mut client_builder = reqwest::Client::builder()
            .timeout(timeout);

        // Ajouter le header X-MBX-APIKEY si un token est fourni
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
            .expect("Impossible de créer le client HTTP");

        Self {
            client,
            base_url: BINANCE_API_BASE.to_string(),
            api_token,
        }
    }

    /// Récupère la dernière bougie de manière asynchrone (pour Iced Tasks)
    ///
    /// Cette méthode permet de faire des requêtes en parallèle sans bloquer le thread principal.
    pub async fn get_latest_candle_async(&self, series_id: &SeriesId) -> Result<Option<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let candles = self.fetch_klines(&symbol, &interval, None, None, Some(1)).await?;
        Ok(candles.into_iter().last())
    }

    /// Récupère les nouvelles bougies depuis un timestamp de manière asynchrone (pour Iced Tasks)
    ///
    /// Cette méthode permet de faire des requêtes en parallèle sans bloquer le thread principal.
    pub async fn fetch_new_candles_async(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, ProviderError> {
        let (symbol, interval) = self.parse_series_id(series_id)?;
        let start_time_ms = since_timestamp * 1000;
        self.fetch_klines(&symbol, &interval, Some(start_time_ms), None, Some(1000)).await
    }

    /// Récupère toutes les bougies de manière asynchrone (pour Iced Tasks)
    ///
    /// Cette méthode permet de faire des requêtes en parallèle sans bloquer le thread principal.
    pub async fn fetch_all_candles_async(&self, series_id: &SeriesId) -> Result<Vec<Candle>, ProviderError> {
        self.fetch_new_candles_async(series_id, 0).await
    }

    /// Récupère les bougies dans une plage temporelle spécifique de manière asynchrone (pour Iced Tasks)
    ///
    /// Cette méthode permet de récupérer les bougies manquantes pour combler un gap.
    ///
    /// # Arguments
    /// * `series_id` - Identifiant de la série
    /// * `start_timestamp` - Timestamp de début (en secondes)
    /// * `end_timestamp` - Timestamp de fin (en secondes)
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
    ///
    /// Format attendu: "SYMBOL_INTERVAL" (ex: "BTCUSDT_1h")
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
    ///
    /// L'API Binance retourne les klines sous forme de tableaux :
    /// [open_time (ms), open, high, low, close, volume, close_time, ...]
    fn parse_kline_array(&self, arr: &[serde_json::Value]) -> Result<Candle, ProviderError> {
        if arr.len() < 6 {
            return Err(ProviderError::Parse(format!(
                "Tableau kline incomplet: {} éléments (attendu: au moins 6)",
                arr.len()
            )));
        }

        // Helper pour parser un prix depuis un Value
        let parse_price = |idx: usize, field: &str| -> Result<f64, ProviderError> {
            arr[idx]
                .as_str()
                .ok_or_else(|| ProviderError::Parse(format!("{} invalide (string)", field)))?
                .parse::<f64>()
                .map_err(|e| ProviderError::Parse(format!("Erreur parsing {}: {}", field, e)))
        };

        // Extraire les valeurs
        let open_time_ms = arr[0]
            .as_i64()
            .ok_or_else(|| ProviderError::Parse("open_time invalide".to_string()))?;
        let open = parse_price(1, "open")?;
        let high = parse_price(2, "high")?;
        let low = parse_price(3, "low")?;
        let close = parse_price(4, "close")?;

        // Parser le volume (index 5 dans le tableau Binance)
        let volume = parse_price(5, "volume")?;

        // Convertir timestamp millisecondes → secondes
        let timestamp = open_time_ms / 1000;

        Ok(Candle::new(timestamp, open, high, low, close, volume))
    }

    /// Exécute une future async, en utilisant le runtime existant ou en créant un nouveau
    ///
    /// Cette fonction gère correctement les appels depuis des contextes sync et async :
    /// - Si `Handle::try_current()` réussit, on pourrait être dans un contexte sync ou async
    /// - Comme `Handle::block_on()` panique si appelé depuis un contexte async, on crée
    ///   toujours un nouveau runtime pour éviter ce risque
    /// - Cette approche est plus sûre même si légèrement moins efficace
    #[allow(dead_code)] // Utilisé dans les implémentations de RealtimeDataProvider (fetch_latest_candle, fetch_new_candles)
    fn run_async<F, T>(&self, future: F) -> Result<T, ProviderError>
    where
        F: std::future::Future<Output = Result<T, ProviderError>>,
    {
        // Toujours créer un nouveau runtime pour éviter les panics
        // Handle::block_on() panique si appelé depuis un contexte async, et il n'y a pas
        // de moyen fiable de détecter si on est dans un contexte async avant d'appeler block_on.
        // Créer un nouveau runtime est la solution la plus sûre.
        tokio::runtime::Runtime::new()
            .map_err(|e| ProviderError::Unknown(format!("Erreur création runtime: {}", e)))?
            .block_on(future)
    }

    /// Récupère les klines depuis l'API Binance
    ///
    /// # Arguments
    /// * `symbol` - Symbole de la paire (ex: "BTCUSDT")
    /// * `interval` - Intervalle (ex: "1h", "15m", "1d")
    /// * `start_time` - Timestamp de début (optionnel, en millisecondes)
    /// * `end_time` - Timestamp de fin (optionnel, en millisecondes)
    /// * `limit` - Nombre maximum de klines à récupérer (max: 1000)
    async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<Candle>, ProviderError> {
        // Construire l'URL de manière optimisée
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

        // Faire la requête
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

        // Parser la réponse JSON
        let json: Vec<Vec<serde_json::Value>> = response
            .json()
            .await
            .map_err(ProviderError::from)?;

        // Convertir en Candles
        let mut candles = Vec::new();
        for kline_arr in json {
            match self.parse_kline_array(&kline_arr) {
                Ok(candle) => candles.push(candle),
                Err(e) => {
                    eprintln!("⚠️ Erreur parsing kline: {}", e);
                    // Continuer avec les autres klines
                }
            }
        }

        Ok(candles)
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
        
        // Format Binance: [open_time_ms, open, high, low, close, volume, ...]
        let kline_json = serde_json::json!([
            1609459200000i64,  // open_time (ms)
            "50000.0",          // open
            "50100.0",          // high
            "49900.0",          // low
            "50050.0",          // close
            "100.5",            // volume
            1609462800000i64,  // close_time
            "50000000.0",       // quote_volume
            1000i64,            // trades
            "2500000.0",        // taker_buy_base
            "125000000.0"       // taker_buy_quote
        ]);
        
        let arr: Vec<serde_json::Value> = serde_json::from_value(kline_json).unwrap();
        let candle = provider.parse_kline_array(&arr).unwrap();
        
        assert_eq!(candle.timestamp, 1609459200); // secondes
        assert_eq!(candle.open, 50000.0);
        assert_eq!(candle.high, 50100.0);
        assert_eq!(candle.low, 49900.0);
        assert_eq!(candle.close, 50050.0);
    }
}

