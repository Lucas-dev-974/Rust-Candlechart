# Système Real-Time - Guide Complet

## 📋 Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Architecture](#architecture)
3. [UpdateResult](#updateresult)
4. [Méthodes de mise à jour](#méthodes-de-mise-à-jour)
5. [Intégration avec Iced](#intégration-avec-iced)
6. [Auto-scroll](#auto-scroll)
7. [Exemples d'utilisation](#exemples-dutilisation)
8. [Bonnes pratiques](#bonnes-pratiques)

---

## Vue d'ensemble

Le système de mise à jour en temps réel permet d'intégrer des sources de données externes (API, WebSocket, etc.) avec le graphique pour des mises à jour automatiques des bougies.

### Fonctionnalités

- ✅ Mise à jour automatique des bougies existantes (bougie courante non fermée)
- ✅ Ajout de nouvelles bougies
- ✅ Synchronisation complète ou partielle des séries
- ✅ Gestion des erreurs et validation des données
- ✅ Parallélisation des requêtes (non-bloquant)
- ✅ Auto-scroll configurable

---

## Architecture

### Flux de données

```
┌─────────────────────────────────────────────────────────┐
│  Iced Framework - Subscription                          │
│  (Toutes les 5 secondes par défaut)                     │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Message::RealtimeUpdate                                │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ChartApp::update_realtime()                            │
│  - Collecte séries actives                              │
│  - Crée Task async avec join_all()                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Provider::get_latest_candle_async() (parallèle)        │
│  - Requêtes HTTP async                                  │
│  - Parsing JSON                                         │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Message::RealtimeUpdateComplete                        │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  ChartApp::apply_realtime_updates()                     │
│  - ChartState::update_candle()                          │
│  - TimeSeries::update_or_append_candle()                │
│  - Invalidation des caches                              │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Auto-scroll (si activé)                                │
│  - Ajuste viewport si proche de la fin                  │
└─────────────────────────────────────────────────────────┘
```

### Composants principaux

1. **RealtimeDataProvider** (Trait) : Interface pour les providers
2. **UpdateResult** (Enum) : Résultat des mises à jour
3. **ChartState** : Méthodes de mise à jour
4. **TimeSeries** : Gestion des bougies (update_or_append, merge)

---

## UpdateResult

L'enum `UpdateResult` représente le résultat d'une mise à jour.

### Définition

```rust
pub enum UpdateResult {
    /// Aucune mise à jour nécessaire
    NoUpdate,
    /// Nouvelle bougie ajoutée
    NewCandle,
    /// Bougie existante mise à jour
    CandleUpdated,
    /// Plusieurs bougies ajoutées
    MultipleCandlesAdded(usize),
    /// Erreur lors de la mise à jour
    Error(String),
}
```

### Utilisation

```rust
match chart_state.update_candle(&series_id, candle) {
    UpdateResult::NewCandle => {
        println!("Nouvelle bougie ajoutée");
    }
    UpdateResult::CandleUpdated => {
        println!("Bougie mise à jour");
    }
    UpdateResult::MultipleCandlesAdded(n) => {
        println!("{} bougies ajoutées", n);
    }
    UpdateResult::NoUpdate => {
        println!("Aucune mise à jour");
    }
    UpdateResult::Error(e) => {
        eprintln!("Erreur: {}", e);
    }
}
```

---

## Méthodes de mise à jour

### Dans ChartState

#### `update_candle()`

Met à jour ou ajoute une bougie à une série spécifique.

```rust
let result = chart_state.update_candle(&series_id, new_candle);
```

**Comportement** :
- Si même timestamp que la dernière bougie → **Met à jour**
- Si nouveau timestamp → **Ajoute** une nouvelle bougie

#### `merge_candles()`

Fusionne plusieurs bougies dans une série (évite les doublons).

```rust
let candles = vec![candle1, candle2, candle3];
let result = chart_state.merge_candles(&series_id, candles);
```

**Comportement** :
- Fusion intelligente avec recherche binaire (O(log n))
- Évite les doublons
- Maintient l'ordre chronologique

#### `sync_from_provider()`

Synchronise une série complète depuis un provider.

```rust
let result = chart_state.sync_from_provider(&series_id, &provider);
```

**Utilisation** : Pour la première connexion ou resynchronisation complète.

#### `fetch_new_candles_from_provider()`

Récupère les nouvelles bougies depuis un timestamp donné.

```rust
let last_ts = chart_state.series_manager
    .get_series(&series_id)
    .and_then(|s| s.data.max_timestamp())
    .unwrap_or(0);

let result = chart_state.fetch_new_candles_from_provider(
    &series_id,
    last_ts,
    &provider
);
```

**Utilisation** : Pour compléter les données manquantes.

### Dans TimeSeries

#### `update_or_append_candle()`

Met à jour la dernière bougie si même timestamp, sinon ajoute.

```rust
match time_series.update_or_append_candle(candle) {
    Ok(true) => println!("Bougie mise à jour"),
    Ok(false) => println!("Nouvelle bougie ajoutée"),
    Err(e) => eprintln!("Erreur: {}", e),
}
```

#### `merge_candles()`

Fusionne plusieurs bougies en évitant les doublons.

```rust
let added_count = time_series.merge_candles(new_candles);
println!("{} bougies ajoutées", added_count);
```

---

## Intégration avec Iced

### Architecture async (non-bloquant)

Le système utilise **Iced Tasks** pour faire les requêtes en parallèle sans bloquer le thread principal.

#### Dans votre application

```rust
use iced::{Task, Subscription};
use std::time::Duration;

struct ChartApp {
    chart_state: ChartState,
    binance_provider: BinanceProvider,
    realtime_enabled: bool,
}

impl ChartApp {
    fn subscription(&self) -> Subscription<Message> {
        if self.realtime_enabled {
            Subscription::batch(vec![
                iced::time::every(Duration::from_secs(5))
                    .map(|_| Message::RealtimeUpdate),
            ])
        } else {
            Subscription::batch(vec![])
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RealtimeUpdate => {
                self.update_realtime()  // Retourne une Task
            }
            Message::RealtimeUpdateComplete(results) => {
                self.apply_realtime_updates(results);
                Task::none()
            }
            // ...
        }
    }

    fn update_realtime(&mut self) -> Task<Message> {
        // Collecter les séries actives
        let active_series: Vec<_> = /* ... */;
        
        // Cloner le provider
        let provider = self.binance_provider.clone();
        
        // Créer une Task async
        Task::perform(
            async move {
                use futures::future::join_all;
                
                // Créer les futures pour toutes les requêtes
                let futures: Vec<_> = active_series
                    .iter()
                    .map(|(series_id, _)| {
                        let provider = provider.clone();
                        let series_id = series_id.clone();
                        async move {
                            let result = provider.get_latest_candle_async(&series_id).await;
                            (series_id, result)
                        }
                    })
                    .collect();
                
                // Exécuter en parallèle
                join_all(futures).await
            },
            Message::RealtimeUpdateComplete,
        )
    }

    fn apply_realtime_updates(&mut self, results: Vec<...>) {
        for (series_id, result) in results {
            if let Ok(Some(candle)) = result {
                chart_state.update_candle(&series_id, candle);
            }
        }
        
        // Auto-scroll si nécessaire
        if self.chart_style.auto_scroll_enabled {
            self.chart_state.auto_scroll_to_latest();
        }
    }
}
```

### Avantages

- ✅ **Non-bloquant** : L'UI reste responsive pendant les requêtes
- ✅ **Parallélisation** : Toutes les requêtes se font en parallèle
- ✅ **Thread-safe** : Iced gère la synchronisation

---

## Auto-scroll

L'auto-scroll ajuste automatiquement le viewport pour afficher les dernières bougies.

### Fonctionnement

```rust
pub fn auto_scroll_to_latest(&mut self) {
    if let Some(active_series) = self.series_manager.active_series().next() {
        if let Some(max_time) = active_series.data.max_timestamp() {
            let (current_min, current_max) = self.viewport.time_scale().time_range();
            let range = current_max - current_min;
            
            // Si on est dans les 10% de la fin, ajuster pour suivre
            if max_time > current_max - (range / 10) {
                self.viewport.focus_on_recent(&active_series.data, DEFAULT_VISIBLE_CANDLES);
            }
        }
    }
}
```

### Configuration

L'auto-scroll peut être désactivé dans les paramètres :

```rust
// Dans ChartStyle
pub struct ChartStyle {
    // ...
    pub auto_scroll_enabled: bool,  // Par défaut: true
}
```

### Logique

- ✅ **Actif** : Si l'utilisateur est dans les 10% de la fin du graphique
- ❌ **Inactif** : Si l'utilisateur regarde une zone plus ancienne

**Raison** : Ne pas perturber l'utilisateur s'il consulte des données historiques.

---

## Exemples d'utilisation

### Exemple 1 : Mise à jour périodique simple

```rust
use candlechart::{ChartState, BinanceProvider, UpdateResult};
use candlechart::core::SeriesId;
use std::time::Duration;
use tokio::time::interval;

async fn update_loop(chart_state: &mut ChartState) {
    let provider = BinanceProvider::new();
    let series_id = SeriesId::new("BTCUSDT_1h");
    let mut update_interval = interval(Duration::from_secs(5));
    
    loop {
        update_interval.tick().await;
        
        match chart_state.update_from_provider(&series_id, &provider) {
            UpdateResult::NewCandle | UpdateResult::CandleUpdated => {
                println!("Graphique mis à jour");
            }
            UpdateResult::Error(e) => {
                eprintln!("Erreur: {}", e);
            }
            _ => {}
        }
    }
}
```

### Exemple 2 : Synchronisation initiale

```rust
fn initialize_series(chart_state: &mut ChartState, provider: &BinanceProvider) {
    let series_id = SeriesId::new("BTCUSDT_1h");
    
    // Synchroniser toutes les bougies
    match chart_state.sync_from_provider(&series_id, provider) {
        UpdateResult::MultipleCandlesAdded(n) => {
            println!("✅ {} bougies chargées", n);
        }
        UpdateResult::Error(e) => {
            eprintln!("❌ Erreur: {}", e);
        }
        _ => {}
    }
    
    // Ajuster le viewport pour afficher les dernières données
    chart_state.auto_scroll_to_latest();
}
```

### Exemple 3 : Complétion des données manquantes

```rust
fn complete_missing_data(chart_state: &mut ChartState, provider: &BinanceProvider) {
    for series in chart_state.series_manager.all_series() {
        let series_id = series.id.clone();
        
        if let Some(last_ts) = series.data.max_timestamp() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            
            // Si données récentes (< 2h), compléter depuis le dernier timestamp
            // Sinon, récupérer les 100 dernières bougies
            let since_ts = if now - last_ts < 7200 {
                last_ts
            } else {
                now - 360000  // 100 heures pour 1h
            };
            
            match chart_state.fetch_new_candles_from_provider(
                &series_id,
                since_ts,
                provider
            ) {
                UpdateResult::MultipleCandlesAdded(n) => {
                    println!("✅ {} nouvelles bougies", n);
                }
                UpdateResult::Error(e) => {
                    eprintln!("❌ Erreur: {}", e);
                }
                _ => {}
            }
        }
    }
}
```

### Exemple 4 : WebSocket

```rust
struct WebSocketProvider {
    latest_candles: Arc<Mutex<HashMap<SeriesId, Candle>>>,
}

// Dans votre handler WebSocket
fn on_websocket_message(chart_state: &mut ChartState, message: CandleUpdate) {
    let candle = message.to_candle();
    match chart_state.update_candle(&message.series_id, candle) {
        UpdateResult::NewCandle | UpdateResult::CandleUpdated => {
            // Le graphique sera automatiquement mis à jour
        }
        UpdateResult::Error(e) => {
            eprintln!("Erreur: {}", e);
        }
        _ => {}
    }
}
```

---

## Bonnes pratiques

### 1. Gérer les erreurs

Toujours vérifier les `UpdateResult::Error` :

```rust
match chart_state.update_candle(&series_id, candle) {
    UpdateResult::Error(e) => {
        eprintln!("Erreur: {}", e);
        // Implémenter une logique de retry si nécessaire
    }
    _ => {}
}
```

### 2. Éviter les doublons

Utiliser `merge_candles()` pour fusionner plusieurs bougies :

```rust
// ✅ Bon
chart_state.merge_candles(&series_id, candles);

// ❌ Mauvais (peut créer des doublons)
for candle in candles {
    chart_state.update_candle(&series_id, candle);
}
```

### 3. Suivre les timestamps

Utiliser `get_last_timestamp()` pour savoir depuis quand récupérer :

```rust
let last_ts = chart_state.series_manager
    .get_series(&series_id)
    .and_then(|s| s.data.max_timestamp())
    .unwrap_or(0);

chart_state.fetch_new_candles_from_provider(&series_id, last_ts, &provider);
```

### 4. Intervalle de mise à jour

Choisir un intervalle approprié selon vos besoins :

```rust
// Pour des données très volatiles
iced::time::every(Duration::from_secs(1))

// Pour des données normales
iced::time::every(Duration::from_secs(5))

// Pour des données stables
iced::time::every(Duration::from_secs(30))
```

### 5. Synchronisation initiale

Toujours faire une synchronisation complète au démarrage :

```rust
// Au démarrage de l'application
for series_id in get_all_series_ids() {
    chart_state.sync_from_provider(&series_id, &provider);
}
```

### 6. Validation des données

Les bougies sont automatiquement validées avant insertion :

- ✅ Timestamp valide (> 0)
- ✅ Prix positifs
- ✅ High >= Low
- ✅ Open, High, Low, Close dans la plage [Low, High]

Les bougies invalides sont rejetées avec un log d'avertissement.

---

## Performance

### Complexité algorithmique

- `update_or_append_candle()` : **O(1)** (accès direct à la fin)
- `merge_candles()` : **O(n log m)** où n = nouvelles bougies, m = bougies existantes
- `fetch_latest_candle()` : **O(1)** (1 requête HTTP)
- `fetch_new_candles()` : **O(1)** (1 requête HTTP, peut retourner jusqu'à 1000 bougies)

### Optimisations

- ✅ **Recherche binaire** : `merge_candles()` utilise `binary_search` (O(log n))
- ✅ **Parallélisation** : Requêtes en parallèle avec `join_all()`
- ✅ **Cache invalidation** : Caches invalidés seulement quand nécessaire
- ✅ **Validation efficace** : Validation rapide avant insertion

---

## Dépannage

### Le graphique ne se met pas à jour

1. Vérifier que `realtime_enabled` est `true`
2. Vérifier que la subscription est active
3. Vérifier les logs pour les erreurs
4. Vérifier que `auto_scroll_enabled` n'est pas désactivé (si nécessaire)

### Erreurs de requête

1. Vérifier la connexion réseau
2. Vérifier les limites de rate limiting de l'API
3. Vérifier le format du `SeriesId` (doit être `SYMBOL_INTERVAL`)

### Performance lente

1. Réduire l'intervalle de mise à jour
2. Réduire le nombre de séries actives
3. Vérifier les timeouts des requêtes

---

## Résumé

- ✅ **Système non-bloquant** : Utilise Iced Tasks pour les requêtes async
- ✅ **Parallélisation** : Toutes les requêtes se font en parallèle
- ✅ **Auto-scroll intelligent** : Suit les nouvelles données si proche de la fin
- ✅ **Gestion d'erreurs** : `UpdateResult` pour tous les cas
- ✅ **Performance** : Recherche binaire et validation efficace

Pour plus d'informations, voir :
- `docs/PROVIDERS.md` : Documentation sur les providers
- `examples/realtime_example.rs` : Exemple complet d'utilisation



