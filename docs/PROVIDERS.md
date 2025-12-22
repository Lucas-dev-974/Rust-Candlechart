# Providers - Guide Complet

## 📋 Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Trait RealtimeDataProvider](#trait-realtimedataprovider)
3. [Provider Binance](#provider-binance)
4. [Créer votre propre Provider](#créer-votre-propre-provider)
5. [Exemples d'implémentation](#exemples-dimplémentation)
6. [Bonnes pratiques](#bonnes-pratiques)

---

## Vue d'ensemble

Les **Providers** sont des abstractions qui permettent de récupérer des données de bougies depuis différentes sources (API REST, WebSocket, fichiers, etc.). Ils implémentent le trait `RealtimeDataProvider` pour s'intégrer avec le système de mise à jour en temps réel.

### Architecture

```
┌─────────────────────────────────────────┐
│     ChartState                          │
│  - update_candle()                      │
│  - sync_from_provider()                 │
│  - fetch_new_candles_from_provider()    │
└──────────────┬──────────────────────────┘
               │
               │ utilise
               ▼
┌─────────────────────────────────────────┐
│  RealtimeDataProvider (Trait)          │
│  - fetch_latest_candle()                │
│  - fetch_new_candles()                  │
│  - fetch_all_candles()                  │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
       ▼                ▼
┌─────────────┐  ┌──────────────┐
│ Binance     │  │ Votre        │
│ Provider    │  │ Provider     │
└─────────────┘  └──────────────┘
```

---

## Trait RealtimeDataProvider

Le trait `RealtimeDataProvider` définit l'interface standard pour tous les providers.

### Définition

```rust
pub trait RealtimeDataProvider {
    /// Récupère la dernière bougie pour une série donnée
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String>;

    /// Récupère les nouvelles bougies depuis un timestamp donné
    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String>;

    /// Récupère toutes les bougies (implémentation par défaut)
    fn fetch_all_candles(&self, series_id: &SeriesId) -> Result<Vec<Candle>, String> {
        self.fetch_new_candles(series_id, 0)
    }
}
```

### Méthodes

#### `fetch_latest_candle()`

Récupère la **dernière bougie** (non fermée) pour une série.

**Retour** :
- `Ok(Some(candle))` : Dernière bougie disponible
- `Ok(None)` : Aucune bougie disponible
- `Err(msg)` : Erreur lors de la récupération

**Utilisation** : Pour les mises à jour périodiques en temps réel.

#### `fetch_new_candles()`

Récupère **toutes les bougies** avec un timestamp >= `since_timestamp`.

**Paramètres** :
- `series_id` : Identifiant de la série
- `since_timestamp` : Timestamp de départ (en secondes)

**Retour** :
- `Ok(candles)` : Liste des bougies (peut être vide)
- `Err(msg)` : Erreur lors de la récupération

**Utilisation** : Pour compléter les données manquantes ou synchroniser.

#### `fetch_all_candles()`

Récupère **toutes les bougies** de la série (implémentation par défaut).

**Utilisation** : Pour la synchronisation complète au démarrage.

---

## Provider Binance

Le `BinanceProvider` est l'implémentation fournie pour récupérer des données depuis l'API Binance.

### Installation

Le provider utilise `reqwest` et `tokio` qui sont déjà dans les dépendances du projet.

### Utilisation de base

#### Créer un provider

```rust
use candlechart::BinanceProvider;
use std::time::Duration;

// Avec les paramètres par défaut (timeout: 10s)
let provider = BinanceProvider::new();

// Avec un timeout personnalisé
let provider = BinanceProvider::with_timeout(Duration::from_secs(5));
```

#### Format des SeriesId

Le provider attend un format spécifique pour les `SeriesId` :
- **Format** : `SYMBOL_INTERVAL`
- **Exemples** : `BTCUSDT_1h`, `ETHUSDT_15m`, `BNBUSDT_1d`

#### Intervalles supportés

- **Minutes** : `1m`, `3m`, `5m`, `15m`, `30m`
- **Heures** : `1h`, `2h`, `4h`, `6h`, `8h`, `12h`
- **Jours** : `1d`, `3d`
- **Semaine** : `1w`
- **Mois** : `1M`

### Méthodes publiques

#### `get_latest_candle_async()`

Méthode async pour récupérer la dernière bougie (utilisée avec Iced Tasks).

```rust
let provider = BinanceProvider::new();
let series_id = SeriesId::new("BTCUSDT_1h");

let candle = provider.get_latest_candle_async(&series_id).await?;
```

**Avantages** :
- Non-bloquant
- Peut être utilisée avec `futures::join_all()` pour paralléliser
- Intégration native avec Iced Tasks

### API Binance utilisée

Le provider utilise l'endpoint public de Binance :
- **GET /api/v3/klines** : Récupère les klines (bougies)

**Paramètres** :
- `symbol` : Symbole de la paire (ex: "BTCUSDT")
- `interval` : Intervalle (ex: "1h", "15m")
- `startTime` : Timestamp de début (optionnel, en millisecondes)
- `endTime` : Timestamp de fin (optionnel, en millisecondes)
- `limit` : Nombre maximum de klines (max: 1000)

**Documentation officielle** : https://binance-docs.github.io/apidocs/spot/en/#kline-candlestick-data

### Limitations

- **Rate limiting** : L'API Binance a des limites (1200 requêtes/minute par IP)
- **Timeout** : Par défaut 10 secondes, ajustable
- **Pagination** : Maximum 1000 bougies par requête

---

## Créer votre propre Provider

### Structure de base

```rust
use candlechart::{RealtimeDataProvider, core::{SeriesId, Candle}};

pub struct MyApiProvider {
    api_client: MyApiClient,
    base_url: String,
}

impl MyApiProvider {
    pub fn new() -> Self {
        Self {
            api_client: MyApiClient::new(),
            base_url: "https://api.example.com".to_string(),
        }
    }
}

impl RealtimeDataProvider for MyApiProvider {
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        // Votre logique ici
        Ok(Some(self.api_client.get_latest_candle(series_id)?))
    }

    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String> {
        // Votre logique ici
        Ok(self.api_client.get_candles_since(series_id, since_timestamp)?)
    }
}
```

### Étapes détaillées

#### 1. Définir la structure

```rust
pub struct MyApiProvider {
    client: reqwest::Client,
    api_key: Option<String>,  // Si nécessaire
    base_url: String,
}
```

#### 2. Implémenter les constructeurs

```rust
impl MyApiProvider {
    pub fn new() -> Self {
        Self::with_config("https://api.example.com", None)
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self::with_config("https://api.example.com", Some(api_key))
    }

    fn with_config(base_url: &str, api_key: Option<String>) -> Self {
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(10));

        // Ajouter des headers si nécessaire
        if let Some(key) = &api_key {
            // Configurer l'authentification
        }

        Self {
            client: client_builder.build().unwrap(),
            api_key,
            base_url: base_url.to_string(),
        }
    }
}
```

#### 3. Implémenter le trait

```rust
impl RealtimeDataProvider for MyApiProvider {
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        // 1. Parser le SeriesId pour extraire les informations nécessaires
        let (symbol, interval) = self.parse_series_id(series_id)?;
        
        // 2. Construire l'URL de l'API
        let url = format!("{}/latest?symbol={}&interval={}", 
                         self.base_url, symbol, interval);
        
        // 3. Faire la requête HTTP
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Erreur HTTP: {}", e))?;
        
        // 4. Parser la réponse
        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Erreur parsing JSON: {}", e))?;
        
        // 5. Convertir en Candle
        let candle = self.parse_response_to_candle(&json)?;
        
        Ok(Some(candle))
    }

    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String> {
        // Logique similaire mais pour plusieurs bougies
        // ...
        Ok(candles)
    }
}
```

#### 4. Helpers utiles

```rust
impl MyApiProvider {
    /// Parse le SeriesId pour extraire les informations
    fn parse_series_id(&self, series_id: &SeriesId) -> Result<(String, String), String> {
        // Votre logique de parsing
        // Exemple: "BTCUSDT_1h" -> ("BTCUSDT", "1h")
    }

    /// Convertit la réponse de l'API en Candle
    fn parse_response_to_candle(&self, json: &serde_json::Value) -> Result<Candle, String> {
        // Votre logique de conversion
        // Assurez-vous de convertir les timestamps correctement
    }
}
```

---

## Exemples d'implémentation

### Exemple 1 : Provider pour API REST simple

```rust
use candlechart::{RealtimeDataProvider, core::{SeriesId, Candle}};
use reqwest::Client;
use std::time::Duration;

pub struct SimpleApiProvider {
    client: Client,
}

impl SimpleApiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

impl RealtimeDataProvider for SimpleApiProvider {
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        let url = format!("https://api.example.com/candles/latest/{}", series_id.name);
        
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Erreur HTTP: {}", e))?;
        
        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Erreur parsing: {}", e))?;
        
        let candle = Candle::new(
            json["timestamp"].as_i64().ok_or("Timestamp invalide")? / 1000,
            json["open"].as_f64().ok_or("Open invalide")?,
            json["high"].as_f64().ok_or("High invalide")?,
            json["low"].as_f64().ok_or("Low invalide")?,
            json["close"].as_f64().ok_or("Close invalide")?,
        );
        
        Ok(Some(candle))
    }

    fn fetch_new_candles(&self, series_id: &SeriesId, since_timestamp: i64) -> Result<Vec<Candle>, String> {
        let url = format!(
            "https://api.example.com/candles/{}?since={}",
            series_id.name, since_timestamp * 1000
        );
        
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Erreur HTTP: {}", e))?;
        
        let json: Vec<serde_json::Value> = response
            .json()
            .map_err(|e| format!("Erreur parsing: {}", e))?;
        
        let mut candles = Vec::new();
        for item in json {
            candles.push(Candle::new(
                item["timestamp"].as_i64().unwrap() / 1000,
                item["open"].as_f64().unwrap(),
                item["high"].as_f64().unwrap(),
                item["low"].as_f64().unwrap(),
                item["close"].as_f64().unwrap(),
            ));
        }
        
        Ok(candles)
    }
}
```

### Exemple 2 : Provider avec authentification

```rust
pub struct AuthenticatedApiProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AuthenticatedApiProvider {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-API-Key",
            reqwest::header::HeaderValue::from_str(&api_key).unwrap(),
        );

        Self {
            client: Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.example.com".to_string(),
        }
    }
}

impl RealtimeDataProvider for AuthenticatedApiProvider {
    // Implémentation similaire mais avec authentification
    // ...
}
```

### Exemple 3 : Provider pour WebSocket (avec buffer)

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WebSocketProvider {
    // Buffer des dernières bougies reçues via WebSocket
    latest_candles: Arc<Mutex<HashMap<SeriesId, Candle>>>,
}

impl WebSocketProvider {
    pub fn new() -> Self {
        // Démarrer la connexion WebSocket dans un thread séparé
        // et mettre à jour latest_candles
        
        Self {
            latest_candles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Méthode appelée par le handler WebSocket
    pub fn on_candle_update(&self, series_id: SeriesId, candle: Candle) {
        let mut candles = self.latest_candles.lock().unwrap();
        candles.insert(series_id, candle);
    }
}

impl RealtimeDataProvider for WebSocketProvider {
    fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        let candles = self.latest_candles.lock().unwrap();
        Ok(candles.get(series_id).cloned())
    }

    fn fetch_new_candles(&self, _series_id: &SeriesId, _since_timestamp: i64) -> Result<Vec<Candle>, String> {
        // Pour WebSocket, on ne peut récupérer que la dernière bougie
        // Pour les bougies historiques, il faudrait utiliser une API REST
        Ok(vec![])
    }
}
```

---

## Bonnes pratiques

### 1. Gestion des erreurs

Toujours retourner des messages d'erreur descriptifs :

```rust
fn fetch_latest_candle(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
    // ❌ Mauvais
    // Err("Erreur".to_string())
    
    // ✅ Bon
    Err(format!("Erreur API pour {}: {}", series_id.name, error))
}
```

### 2. Conversion des timestamps

Les timestamps doivent être en **secondes** (pas en millisecondes) :

```rust
// Si l'API retourne des timestamps en millisecondes
let timestamp_ms = json["timestamp"].as_i64().unwrap();
let timestamp = timestamp_ms / 1000;  // Convertir en secondes

Candle::new(timestamp, open, high, low, close)
```

### 3. Timeout et retry

Toujours configurer un timeout et envisager un système de retry :

```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))  // Timeout
    .build()?;

// Pour le retry, utiliser une bibliothèque comme `reqwest-retry`
```

### 4. Rate limiting

Respecter les limites de l'API :

```rust
use std::time::{Duration, Instant};

pub struct RateLimitedProvider {
    client: Client,
    last_request: Arc<Mutex<Instant>>,
    min_interval: Duration,
}

impl RateLimitedProvider {
    fn wait_if_needed(&self) {
        let mut last = self.last_request.lock().unwrap();
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            std::thread::sleep(self.min_interval - elapsed);
        }
        *last = Instant::now();
    }
}
```

### 5. Validation des données

Valider les données avant de créer les bougies :

```rust
fn parse_candle(&self, json: &serde_json::Value) -> Result<Candle, String> {
    let open = json["open"].as_f64().ok_or("Open invalide")?;
    let high = json["high"].as_f64().ok_or("High invalide")?;
    let low = json["low"].as_f64().ok_or("Low invalide")?;
    let close = json["close"].as_f64().ok_or("Close invalide")?;
    
    // Valider la cohérence OHLC
    if high < low {
        return Err("High < Low".to_string());
    }
    if open < low || open > high || close < low || close > high {
        return Err("Prix hors de la plage High/Low".to_string());
    }
    
    Ok(Candle::new(timestamp, open, high, low, close))
}
```

### 6. Support async

Pour les providers modernes, prévoir une méthode async :

```rust
impl MyApiProvider {
    pub async fn get_latest_candle_async(&self, series_id: &SeriesId) -> Result<Option<Candle>, String> {
        // Version async pour utilisation avec Iced Tasks
        // ...
    }
}
```

---

## Tests

### Exemple de test unitaire

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_latest_candle() {
        let provider = MyApiProvider::new();
        let series_id = SeriesId::new("BTCUSDT_1h");
        
        match provider.fetch_latest_candle(&series_id) {
            Ok(Some(candle)) => {
                assert!(candle.timestamp > 0);
                assert!(candle.high >= candle.low);
            }
            Ok(None) => {
                // Pas de bougie disponible (normal si marché fermé)
            }
            Err(e) => {
                panic!("Erreur: {}", e);
            }
        }
    }
}
```

---

## Résumé

- ✅ **Trait simple** : Seulement 2 méthodes à implémenter
- ✅ **Flexible** : Supporte API REST, WebSocket, fichiers, etc.
- ✅ **Extensible** : Facile d'ajouter de nouveaux providers
- ✅ **Testable** : Interface claire pour les tests

Pour plus d'informations, voir :
- `docs/REALTIME.md` : Documentation sur le système real-time
- `examples/binance_example.rs` : Exemple complet d'utilisation





