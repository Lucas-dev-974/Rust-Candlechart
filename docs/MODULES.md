# Documentation des Modules

## Table des matières

1. [Core](#core)
2. [Scale](#scale)
3. [Viewport](#viewport)
4. [Render](#render)
5. [Interaction](#interaction)
6. [State](#state)
7. [Widget](#widget)
8. [Data Loader](#data-loader)
9. [Settings](#settings)
10. [Messages](#messages)

---

## Core

**Chemin** : `src/finance_chart/core/`

**Responsabilité** : Modèles de données financières et structures de base

### Modules

#### `candle.rs`

Structure de base pour une bougie OHLC.

```rust
pub struct Candle {
    pub timestamp: i64,  // Unix timestamp en secondes
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
```

**Méthodes principales** :
- `new()` : Crée une nouvelle bougie
- `is_bullish()` : Vérifie si la bougie est haussière

#### `timeseries.rs`

Collection ordonnée de bougies avec cache intégré.

```rust
pub struct TimeSeries {
    candles: Vec<Candle>,
    price_cache: PriceRangeCache,
    time_cache: TimeRangeCache,
}
```

**Méthodes principales** :
- `new()` : Crée une série vide
- `push()` : Ajoute une bougie (invalide le cache)
- `len()` : Nombre de bougies
- `min_timestamp()` / `max_timestamp()` : Plage temporelle (avec cache)
- `price_range()` : Plage de prix globale (avec cache)
- `price_range_for_time_range()` : Plage de prix pour une plage temporelle (avec cache)
- `visible_candles()` : Bougies dans une plage (recherche binaire)

**Optimisations** :
- Cache des plages de prix (global et par plage temporelle)
- Recherche binaire pour `visible_candles()` (O(log n))

#### `series_data.rs`

Gestion de plusieurs séries temporelles.

```rust
pub struct SeriesId {
    pub name: String,  // Ex: "BTCUSDT_1h"
}

pub struct SeriesData {
    pub id: SeriesId,
    pub data: TimeSeries,
    pub symbol: String,      // Ex: "BTCUSDT"
    pub interval: String,    // Ex: "1h"
    pub color: Option<Color>,
}

pub struct SeriesManager {
    series: HashMap<SeriesId, SeriesData>,
    active_series: Vec<SeriesId>,
}
```

**Méthodes principales** :
- `add_series()` : Ajoute une série (active la première automatiquement)
- `get_series()` : Récupère une série par ID
- `all_series()` : Itérateur sur toutes les séries
- `active_series()` : Itérateur sur les séries actives
- `activate_only_series()` : Active une seule série (désactive les autres)
- `visible_candles()` : Bougies visibles de toutes les séries actives

#### `cache.rs`

Système de cache pour optimiser les calculs de plages.

```rust
pub struct PriceRangeCache {
    global_price_range: Cell<Option<(f64, f64)>>,
    range_cache: RefCell<HashMap<(i64, i64), (f64, f64)>>,
}

pub struct TimeRangeCache {
    global_time_range: Cell<Option<(i64, i64)>>,
}
```

**Fonctionnalités** :
- Cache global pour toute la série
- Cache par plage temporelle (limité à 100 entrées)
- Invalidation automatique lors des modifications

---

## Scale

**Chemin** : `src/finance_chart/scale/`

**Responsabilité** : Conversion entre valeurs de données et coordonnées écran

### `price.rs`

Échelle linéaire pour les prix.

```rust
pub struct PriceScale {
    min_price: f64,
    max_price: f64,
    height: f32,
    margin_ratio: f32,  // 10% par défaut
}
```

**Méthodes principales** :
- `price_to_y()` : Convertit un prix en coordonnée Y (0 = haut)
- `y_to_price()` : Convertit une coordonnée Y en prix
- `set_price_range()` : Met à jour la plage de prix
- `price_range()` : Retourne la plage actuelle

**Caractéristiques** :
- Marges automatiques (10% par défaut)
- Y inversé (0 = haut, height = bas)

### `time.rs`

Échelle linéaire pour les timestamps.

```rust
pub struct TimeScale {
    min_time: i64,
    max_time: i64,
    width: f32,
}
```

**Méthodes principales** :
- `time_to_x()` : Convertit un timestamp en coordonnée X
- `x_to_time()` : Convertit une coordonnée X en timestamp
- `set_time_range()` : Met à jour la plage temporelle
- `time_range()` : Retourne la plage actuelle

---

## Viewport

**Chemin** : `src/finance_chart/viewport/`

**Responsabilité** : Gestion de la vue visible du graphique

```rust
pub struct Viewport {
    price_scale: PriceScale,
    time_scale: TimeScale,
    width: f32,
    height: f32,
}
```

**Méthodes principales** :
- `new()` : Crée un nouveau viewport
- `focus_on_recent()` : Initialise sur les N dernières bougies
- `zoom()` / `zoom_horizontal()` : Zoom sur l'axe X
- `zoom_vertical()` : Zoom sur l'axe Y
- `zoom_both()` : Zoom sur les deux axes
- `pan_horizontal()` : Déplacement horizontal
- `pan_vertical()` : Déplacement vertical
- `set_size()` : Met à jour la taille

**Limites** :
- `MIN_TIME_RANGE` : 60 secondes
- `MAX_TIME_RANGE` : 1 an
- `MIN_PRICE_RANGE` : 0.01
- `MAX_PRICE_RANGE` : 1 000 000

---

## Render

**Chemin** : `src/finance_chart/render/`

**Responsabilité** : Fonctions de rendu des éléments graphiques

### Modules

#### `candlestick.rs`

Rendu des bougies OHLC.

**Fonctions** :
- `render_candlestick()` : Rend une bougie individuelle
- `render_candlesticks()` : Rend toutes les bougies visibles

**Caractéristiques** :
- Largeur adaptative selon le zoom
- Couleurs personnalisables (bullish/bearish/wick)
- Support de plusieurs séries avec couleurs différentes

#### `grid.rs`

Rendu de la grille.

**Fonctions** :
- `render_grid()` : Rend la grille (lignes horizontales et verticales)
- `calculate_nice_step()` : Calcule un pas "nice" pour les prix
- `calculate_nice_time_step()` : Calcule un pas "nice" pour les temps
- `format_time()` : Formate un timestamp pour affichage

**Algorithme "nice"** : Choisit des valeurs arrondies (1, 2, 5, 10, 20, 50, etc.)

#### `crosshair.rs`

Rendu du crosshair (suivi de la souris).

**Fonctions** :
- `render_crosshair()` : Rend le crosshair avec labels de prix/temps

#### `tooltip.rs`

Rendu du tooltip OHLC (affiché avec SHIFT).

**Fonctions** :
- `render_tooltip()` : Rend le tooltip avec les valeurs OHLC
- `find_candle_at_position()` : Trouve la bougie sous la souris

#### `rectangles.rs`

Rendu des rectangles dessinés.

**Fonctions** :
- `draw_rectangle()` : Rend un rectangle
- `draw_preview_rectangle()` : Rend l'aperçu pendant le dessin

#### `horizontal_line.rs`

Rendu des lignes horizontales.

**Fonctions** :
- `draw_horizontal_line()` : Rend une ligne horizontale
- `draw_hline_preview()` : Rend l'aperçu pendant le dessin
- `hit_test_hline()` : Teste si la souris est sur une ligne

#### `current_price.rs`

Rendu de la ligne de prix courant.

**Fonctions** :
- `render_current_price_line()` : Rend la ligne du dernier prix

---

## Interaction

**Chemin** : `src/finance_chart/interaction/`

**Responsabilité** : Gestion des interactions utilisateur

### `events.rs`

État des interactions.

```rust
pub struct InteractionState {
    pub mouse_position: Option<Point>,
    pub drag_start: Option<Point>,
    pub is_panning: bool,
}
```

**Méthodes** :
- `start_pan()` : Démarre un pan
- `update_pan()` : Met à jour le pan (retourne le delta)
- `end_pan()` : Termine le pan

### `rectangle_editing.rs`

Logique d'édition des rectangles.

**Fonctions** :
- `hit_test_rectangles()` : Trouve le rectangle sous la souris
- `cursor_for_edit_mode()` : Retourne le curseur approprié
- `apply_edit_update()` : Applique une modification de rectangle

**Modes d'édition** :
- `Move` : Déplacement
- `ResizeTopLeft`, `ResizeTopRight`, etc. : Redimensionnement depuis un coin
- `ResizeTop`, `ResizeBottom`, etc. : Redimensionnement depuis un bord

---

## State

**Chemin** : `src/finance_chart/state/`

**Responsabilité** : État global du graphique

```rust
pub struct ChartState {
    pub series_manager: SeriesManager,
    pub viewport: Viewport,
    pub interaction: InteractionState,
}
```

**Méthodes principales** :
- `new()` : Crée un nouvel état
- `add_series()` : Ajoute une série
- `update_viewport_from_series()` : Met à jour le viewport
- `visible_candles()` : Bougies visibles
- `last_candle()` : Dernière bougie de la série active
- `pan_horizontal()` / `pan_vertical()` : Pan
- `zoom()` / `zoom_vertical()` / `zoom_both()` : Zoom
- `start_pan()` / `update_pan()` / `end_pan()` : Gestion du pan

---

## Widget

**Chemin** : `src/finance_chart/widget.rs`

**Responsabilité** : Widget canvas principal implémentant `Program<ChartMessage>`

```rust
pub struct ChartProgram<'a> {
    chart_state: &'a ChartState,
    tools_state: &'a ToolsState,
    settings_state: &'a SettingsState,
    chart_style: &'a ChartStyle,
}

pub struct WidgetState {
    pub alt_pressed: bool,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
}
```

**Méthodes principales** :
- `draw()` : Rendu du graphique
- `update()` : Gestion des événements
- `handle_key_press()` : Gestion des touches clavier
- `handle_mouse_press()` : Gestion du clic souris
- `handle_mouse_move()` : Gestion du mouvement souris
- `handle_scroll()` : Gestion de la molette

**Rendu** :
1. Fond
2. Grille
3. Bougies
4. Ligne de prix courant
5. Dessins (rectangles, lignes)
6. Crosshair (si souris présente)
7. Tooltip (si SHIFT maintenu)

---

## Data Loader

**Chemin** : `src/finance_chart/data_loader.rs`

**Responsabilité** : Chargement des données depuis fichiers JSON

**Fonctions** :
- `load_from_json()` : Charge une série depuis un fichier JSON
- `load_all_from_directory()` : Charge toutes les séries d'un dossier

**Format JSON attendu** :
```json
{
  "symbol": "BTCUSDT",
  "interval": "1h",
  "klines": [
    {
      "open_time": 1609459200000,
      "open": 29374.15,
      "high": 29380.00,
      "low": 29350.00,
      "close": 29360.00,
      "volume": 123.45
    }
  ]
}
```

**Conversion** :
- Timestamps : millisecondes → secondes
- Extraction du symbole et de l'intervalle depuis le JSON

---

## Settings

**Chemin** : `src/finance_chart/settings.rs`

**Responsabilité** : Configuration et styles du graphique

```rust
pub struct ChartStyle {
    pub background_color: SerializableColor,
    pub bullish_color: SerializableColor,
    pub bearish_color: SerializableColor,
    pub wick_color: SerializableColor,
    pub grid_color: SerializableColor,
    pub current_price_color: SerializableColor,
    pub crosshair_color: SerializableColor,
    pub text_color: SerializableColor,
}

pub struct SerializableColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```

**Méthodes** :
- `save_to_file()` : Sauvegarde dans `chart_style.json`
- `load_from_file()` : Charge depuis `chart_style.json`
- `default()` : Style par défaut

**Fonctions utilitaires** :
- `color_fields()` : Liste des champs de couleur éditables
- `preset_colors()` : Couleurs prédéfinies

---

## Messages

**Chemin** : `src/finance_chart/messages.rs`

**Responsabilité** : Types de messages pour la communication

### `ChartMessage`

Messages du canvas principal :
- Navigation : `StartPan`, `UpdatePan`, `EndPan`, `ZoomHorizontal`, `ZoomVertical`, `ZoomBoth`
- Dessin : `StartDrawingRectangle`, `UpdateDrawing`, `FinishDrawingRectangle`, `StartDrawingHLine`, `FinishDrawingHLine`, `CancelDrawing`
- Édition : `StartRectangleEdit`, `UpdateRectangleEdit`, `FinishRectangleEdit`, `DeselectRectangle`, `StartHLineEdit`, `UpdateHLineEdit`, `FinishHLineEdit`, `DeselectHLine`
- Suppression : `DeleteSelected`
- Historique : `Undo`, `Redo`
- Persistance : `SaveDrawings`, `LoadDrawings`
- Utilitaires : `MouseMoved`, `Resize`

### `YAxisMessage` / `XAxisMessage`

Messages des axes pour le zoom par drag.

### `ToolsPanelMessage`

Messages du panel d'outils :
- `ToggleTool` : Active/désactive un outil

### `SeriesPanelMessage`

Messages de sélection de série :
- `SelectSeriesByName` : Sélectionne une série par son nom

---

## Dépendances entre modules

```
main.rs
    │
    └─► finance_chart/
            │
            ├─► core (indépendant)
            │
            ├─► scale (utilise core)
            │
            ├─► viewport (utilise scale, core)
            │
            ├─► state (utilise viewport, core, interaction)
            │
            ├─► render (utilise viewport, core)
            │
            ├─► interaction (utilise core)
            │
            ├─► widget (utilise state, render, interaction, tools_canvas, settings)
            │
            ├─► data_loader (utilise core)
            │
            └─► settings (indépendant)
```

