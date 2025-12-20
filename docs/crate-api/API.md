# Référence API

## Table des matières

1. [API Publique](#api-publique)
2. [Core API](#core-api)
3. [Viewport API](#viewport-api)
4. [Render API](#render-api)
5. [State API](#state-api)

---

## API Publique

### Module `finance_chart`

#### Types exportés

```rust
pub use state::ChartState;
pub use widget::chart;
pub use data_loader::{load_from_json, load_all_from_directory};
pub use axis_canvas::{x_axis, y_axis, X_AXIS_HEIGHT, Y_AXIS_WIDTH};
pub use tools_canvas::ToolsState;
pub use tools_panel_canvas::{tools_panel, TOOLS_PANEL_WIDTH};
pub use series_select::series_select_box;
pub use settings::{ChartStyle, SettingsState};
pub use messages::{ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage};
```

#### Fonctions

##### `load_from_json<P: AsRef<Path>>(path: P) -> Result<SeriesData, LoadError>`

Charge une série depuis un fichier JSON.

**Paramètres** :
- `path` : Chemin vers le fichier JSON

**Retour** :
- `Ok(SeriesData)` : Série chargée avec succès
- `Err(LoadError)` : Erreur de chargement

**Exemple** :
```rust
let series = load_from_json("data/BTCUSDT_1h.json")?;
```

##### `load_all_from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Vec<SeriesData>, LoadError>`

Charge toutes les séries depuis un dossier.

**Paramètres** :
- `dir_path` : Chemin vers le dossier contenant les fichiers JSON

**Retour** :
- `Ok(Vec<SeriesData>)` : Liste de toutes les séries chargées
- `Err(LoadError)` : Erreur de chargement

**Exemple** :
```rust
let series_list = load_all_from_directory("data")?;
```

##### `chart(chart_state, tools_state, settings_state, chart_style) -> Element<ChartMessage>`

Crée le widget canvas principal du graphique.

**Paramètres** :
- `chart_state: &ChartState` : État du graphique
- `tools_state: &ToolsState` : État des outils
- `settings_state: &SettingsState` : État des settings
- `chart_style: &ChartStyle` : Style du graphique

**Retour** : `Element<ChartMessage>`

---

## Core API

### `Candle`

```rust
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
```

#### Méthodes

##### `new(timestamp: i64, open: f64, high: f64, low: f64, close: f64) -> Self`

Crée une nouvelle bougie.

##### `is_bullish(&self) -> bool`

Retourne `true` si la bougie est haussière (close > open).

---

### `TimeSeries`

```rust
pub struct TimeSeries {
    // Champs privés
}
```

#### Méthodes

##### `new() -> Self`

Crée une nouvelle série temporelle vide.

##### `push(&mut self, candle: Candle)`

Ajoute une bougie à la série. Invalide automatiquement les caches.

##### `len(&self) -> usize`

Retourne le nombre de bougies dans la série.

##### `min_timestamp(&self) -> Option<i64>`

Retourne le timestamp minimum (utilise le cache).

##### `max_timestamp(&self) -> Option<i64>`

Retourne le timestamp maximum (utilise le cache).

##### `last_candle(&self) -> Option<&Candle>`

Retourne la dernière bougie (prix courant).

##### `price_range(&self) -> Option<(f64, f64)>`

Retourne la plage de prix (min, max) de toute la série (utilise le cache).

##### `price_range_for_time_range(&self, time_range: Range<i64>) -> Option<(f64, f64)>`

Retourne la plage de prix pour une plage temporelle spécifique (utilise le cache).

##### `visible_candles(&self, time_range: Range<i64>) -> &[Candle]`

Retourne les bougies visibles dans une plage de timestamps (recherche binaire).

---

### `SeriesManager`

```rust
pub struct SeriesManager {
    // Champs privés
}
```

#### Méthodes

##### `new() -> Self`

Crée un nouveau gestionnaire de séries.

##### `add_series(&mut self, series: SeriesData)`

Ajoute une série. Active automatiquement la première série ajoutée.

##### `get_series(&self, id: &SeriesId) -> Option<&SeriesData>`

Récupère une série par son ID.

##### `all_series(&self) -> impl Iterator<Item = &SeriesData>`

Retourne un itérateur sur toutes les séries disponibles.

##### `active_series(&self) -> impl Iterator<Item = &SeriesData>`

Retourne un itérateur sur les séries actives.

##### `activate_only_series(&mut self, id: SeriesId)`

Active uniquement la série spécifiée (désactive toutes les autres).

##### `visible_candles(&self, time_range: Range<i64>) -> Vec<(SeriesId, &[Candle])>`

Retourne toutes les bougies visibles de toutes les séries actives dans une plage temporelle.

##### `total_count(&self) -> usize`

Retourne le nombre total de séries.

---

## Viewport API

### `Viewport`

```rust
pub struct Viewport {
    // Champs privés
}
```

#### Méthodes

##### `new(width: f32, height: f32) -> Self`

Crée un nouveau viewport.

##### `focus_on_recent(&mut self, data: &TimeSeries, visible_candles: usize)`

Initialise le viewport sur les N dernières bougies.

**Paramètres** :
- `data` : La série temporelle
- `visible_candles` : Nombre de bougies à afficher

##### `set_size(&mut self, width: f32, height: f32)`

Met à jour la taille du viewport.

##### `price_scale(&self) -> &PriceScale`

Retourne une référence à l'échelle de prix.

##### `time_scale(&self) -> &TimeScale`

Retourne une référence à l'échelle temporelle.

##### `width(&self) -> f32`

Retourne la largeur du viewport.

##### `height(&self) -> f32`

Retourne la hauteur du viewport.

##### `zoom(&mut self, factor: f64)`

Zoom horizontal (axe X).

**Paramètres** :
- `factor > 1.0` : Zoom out (plage plus grande)
- `factor < 1.0` : Zoom in (plage plus petite)

##### `zoom_vertical(&mut self, factor: f64)`

Zoom vertical (axe Y).

##### `zoom_both(&mut self, factor: f64)`

Zoom sur les deux axes.

##### `pan_horizontal(&mut self, delta_x: f32)`

Pan horizontal basé sur un delta en pixels.

##### `pan_vertical(&mut self, delta_y: f32)`

Pan vertical basé sur un delta en pixels.

---

### `PriceScale`

```rust
pub struct PriceScale {
    // Champs privés
}
```

#### Méthodes

##### `new(min_price: f64, max_price: f64, height: f32) -> Self`

Crée une nouvelle échelle de prix.

##### `set_height(&mut self, height: f32)`

Met à jour la hauteur disponible.

##### `set_price_range(&mut self, min: f64, max: f64)`

Met à jour la plage de prix.

##### `price_range(&self) -> (f64, f64)`

Retourne la plage de prix actuelle.

##### `price_to_y(&self, price: f64) -> f32`

Convertit un prix en coordonnée Y (0 = haut de l'écran).

##### `y_to_price(&self, y: f32) -> f64`

Convertit une coordonnée Y en prix.

---

### `TimeScale`

```rust
pub struct TimeScale {
    // Champs privés
}
```

#### Méthodes

##### `new(min_time: i64, max_time: i64, width: f32) -> Self`

Crée une nouvelle échelle temporelle.

##### `set_width(&mut self, width: f32)`

Met à jour la largeur disponible.

##### `set_time_range(&mut self, min: i64, max: i64)`

Met à jour la plage temporelle.

##### `time_range(&self) -> (i64, i64)`

Retourne la plage temporelle actuelle.

##### `time_to_x(&self, timestamp: i64) -> f32`

Convertit un timestamp en coordonnée X.

##### `x_to_time(&self, x: f32) -> i64`

Convertit une coordonnée X en timestamp.

---

## Render API

### Fonctions de rendu

Toutes les fonctions de rendu prennent un `&mut Frame` et des données, et dessinent directement sur le frame.

#### `render_candlesticks(frame, candles, viewport, colors)`

Rend toutes les bougies visibles.

**Paramètres** :
- `frame: &mut Frame`
- `candles: &[Candle]`
- `viewport: &Viewport`
- `colors: Option<CandleColors>`

#### `render_grid(frame, viewport, style)`

Rend la grille.

**Paramètres** :
- `frame: &mut Frame`
- `viewport: &Viewport`
- `style: Option<GridStyle>`

#### `render_crosshair(frame, viewport, position, style)`

Rend le crosshair.

**Paramètres** :
- `frame: &mut Frame`
- `viewport: &Viewport`
- `position: Point`
- `style: Option<CrosshairStyle>`

#### `render_tooltip(frame, candle, position, viewport, style)`

Rend le tooltip OHLC.

**Paramètres** :
- `frame: &mut Frame`
- `candle: &Candle`
- `position: Point`
- `viewport: &Viewport`
- `style: Option<TooltipStyle>`

#### `render_current_price_line(frame, viewport, price, style)`

Rend la ligne de prix courant.

**Paramètres** :
- `frame: &mut Frame`
- `viewport: &Viewport`
- `price: f64`
- `style: Option<CurrentPriceStyle>`

---

## State API

### `ChartState`

```rust
pub struct ChartState {
    pub series_manager: SeriesManager,
    pub viewport: Viewport,
    pub interaction: InteractionState,
}
```

#### Méthodes

##### `new(width: f32, height: f32) -> Self`

Crée un nouvel état de graphique.

##### `add_series(&mut self, series: SeriesData)`

Ajoute une série au graphique et met à jour le viewport.

##### `update_viewport_from_series(&mut self)`

Met à jour le viewport en fonction des séries actives. Réinitialise le zoom.

##### `resize(&mut self, width: f32, height: f32)`

Met à jour la taille du viewport.

##### `visible_candles(&self) -> Vec<(SeriesId, &[Candle])>`

Retourne les bougies visibles dans le viewport actuel.

##### `last_candle(&self) -> Option<&Candle>`

Retourne la dernière bougie de la première série active.

##### `pan_horizontal(&mut self, delta_x: f32)`

Effectue un pan horizontal.

##### `pan_vertical(&mut self, delta_y: f32)`

Effectue un pan vertical.

##### `zoom(&mut self, factor: f64)`

Effectue un zoom horizontal.

##### `zoom_vertical(&mut self, factor: f64)`

Effectue un zoom vertical.

##### `zoom_both(&mut self, factor: f64)`

Effectue un zoom sur les deux axes.

##### `start_pan(&mut self, position: Point)`

Démarre un pan.

##### `update_pan(&mut self, position: Point)`

Met à jour le pan en cours.

##### `end_pan(&mut self)`

Termine le pan.

---

## Constants

### Dimensions

- `X_AXIS_HEIGHT: f32 = 30.0` : Hauteur de l'axe X
- `Y_AXIS_WIDTH: f32 = 43.0` : Largeur de l'axe Y
- `TOOLS_PANEL_WIDTH: f32` : Largeur du panel d'outils

### Limites de zoom

- `MIN_TIME_RANGE: i64 = 60` : Minimum 1 minute visible
- `MAX_TIME_RANGE: i64 = 365 * 24 * 3600` : Maximum 1 an visible
- `MIN_PRICE_RANGE: f64 = 0.01` : Minimum 0.01 de plage de prix
- `MAX_PRICE_RANGE: f64 = 1_000_000.0` : Maximum 1M de plage de prix

### Cache

- `MAX_CACHE_SIZE: usize = 100` : Taille maximale du cache par plage temporelle

### Historique

- `MAX_HISTORY_SIZE: usize = 50` : Taille maximale de l'historique undo/redo

