# Structures de Données

## Table des matières

1. [Structures Core](#structures-core)
2. [Structures de Scale](#structures-de-scale)
3. [Structures de Viewport](#structures-de-viewport)
4. [Structures de Render](#structures-de-render)
5. [Structures d'Interaction](#structures-dinteraction)
6. [Structures d'État](#structures-détat)
7. [Structures de Tools](#structures-de-tools)
8. [Structures de Settings](#structures-de-settings)

---

## Structures Core

### `Candle`

Représente une bougie OHLC (Open, High, Low, Close).

```rust
pub struct Candle {
    pub timestamp: i64,  // Unix timestamp en secondes
    pub open: f64,       // Prix d'ouverture
    pub high: f64,       // Prix le plus haut
    pub low: f64,        // Prix le plus bas
    pub close: f64,      // Prix de clôture
}
```

**Caractéristiques** :
- `Copy` : Peut être copiée sans allocation
- Triée par `timestamp` dans `TimeSeries`

### `TimeSeries`

Collection ordonnée de bougies avec cache intégré.

```rust
pub struct TimeSeries {
    candles: Vec<Candle>,              // Bougies triées par timestamp
    price_cache: PriceRangeCache,      // Cache des plages de prix
    time_cache: TimeRangeCache,        // Cache des plages temporelles
}
```

**Invariants** :
- Les bougies sont toujours triées par `timestamp` croissant
- Le cache est invalidé lors de `push()`

### `SeriesId`

Identifiant unique d'une série temporelle.

```rust
pub struct SeriesId {
    pub name: String,  // Ex: "BTCUSDT_1h"
}
```

**Caractéristiques** :
- `Hash` : Peut être utilisé comme clé dans `HashMap`
- `PartialEq`, `Eq` : Comparaison par valeur

### `SeriesData`

Données d'une série temporelle avec métadonnées.

```rust
pub struct SeriesData {
    pub id: SeriesId,              // Identifiant unique
    pub data: TimeSeries,          // Données de la série
    pub symbol: String,            // Ex: "BTCUSDT"
    pub interval: String,          // Ex: "1h", "15m", "1d"
    pub color: Option<Color>,      // Couleur personnalisée (optionnel)
}
```

### `SeriesManager`

Gestionnaire de plusieurs séries temporelles.

```rust
pub struct SeriesManager {
    series: HashMap<SeriesId, SeriesData>,  // Toutes les séries
    active_series: Vec<SeriesId>,           // Séries actives (affichées)
}
```

**Invariants** :
- `active_series` ne contient que des IDs présents dans `series`
- Au moins une série est active si `series` n'est pas vide

### `PriceRangeCache`

Cache pour les plages de prix calculées.

```rust
pub struct PriceRangeCache {
    global_price_range: Cell<Option<(f64, f64)>>,  // Cache global
    range_cache: RefCell<HashMap<(i64, i64), (f64, f64)>>,  // Cache par plage
}
```

**Limites** :
- Cache par plage limité à 100 entrées (vidé si dépassé)

### `TimeRangeCache`

Cache pour les plages temporelles calculées.

```rust
pub struct TimeRangeCache {
    global_time_range: Cell<Option<(i64, i64)>>,  // Cache global
}
```

---

## Structures de Scale

### `PriceScale`

Échelle linéaire pour convertir les prix en coordonnées Y.

```rust
pub struct PriceScale {
    min_price: f64,      // Prix minimum visible
    max_price: f64,      // Prix maximum visible
    height: f32,         // Hauteur disponible en pixels
    margin_ratio: f32,    // Marge verticale (10% par défaut)
}
```

**Conversion** :
- `price_to_y()` : Prix → Y (0 = haut, height = bas)
- `y_to_price()` : Y → Prix

### `TimeScale`

Échelle linéaire pour convertir les timestamps en coordonnées X.

```rust
pub struct TimeScale {
    min_time: i64,   // Timestamp minimum visible
    max_time: i64,   // Timestamp maximum visible
    width: f32,      // Largeur disponible en pixels
}
```

**Conversion** :
- `time_to_x()` : Timestamp → X
- `x_to_time()` : X → Timestamp

---

## Structures de Viewport

### `Viewport`

Gère la vue visible du graphique.

```rust
pub struct Viewport {
    price_scale: PriceScale,  // Échelle de prix
    time_scale: TimeScale,    // Échelle temporelle
    width: f32,               // Largeur totale en pixels
    height: f32,              // Hauteur totale en pixels
}
```

**Responsabilités** :
- Zoom horizontal/vertical/les deux
- Pan horizontal/vertical
- Focus sur les données récentes

---

## Structures de Render

### `CandleColors`

Couleurs pour le rendu des bougies.

```rust
pub struct CandleColors {
    pub bullish: Color,   // Couleur bougie haussière
    pub bearish: Color,   // Couleur bougie baissière
    pub wick: Color,      // Couleur des mèches
}
```

### `GridStyle`

Style pour le rendu de la grille.

```rust
pub struct GridStyle {
    pub line_color: Color,
    pub line_width: f32,
}
```

### `CrosshairStyle`

Style pour le rendu du crosshair.

```rust
pub struct CrosshairStyle {
    pub line_color: Color,
    pub label_text_color: Color,
    // ... autres champs
}
```

---

## Structures d'Interaction

### `InteractionState`

État des interactions utilisateur.

```rust
pub struct InteractionState {
    pub mouse_position: Option<Point>,  // Position actuelle de la souris
    pub drag_start: Option<Point>,      // Position de départ d'un drag
    pub is_panning: bool,               // Indique si un pan est en cours
}
```

### `EditMode`

Mode d'édition d'un rectangle.

```rust
pub enum EditMode {
    Move,                    // Déplacement du rectangle entier
    ResizeTopLeft,          // Redimensionnement depuis le coin haut-gauche
    ResizeTopRight,         // Redimensionnement depuis le coin haut-droite
    ResizeBottomLeft,       // Redimensionnement depuis le coin bas-gauche
    ResizeBottomRight,      // Redimensionnement depuis le coin bas-droite
    ResizeTop,              // Redimensionnement depuis le bord haut
    ResizeBottom,           // Redimensionnement depuis le bord bas
    ResizeLeft,             // Redimensionnement depuis le bord gauche
    ResizeRight,            // Redimensionnement depuis le bord droit
}
```

---

## Structures d'État

### `ChartState`

État complet du graphique.

```rust
pub struct ChartState {
    pub series_manager: SeriesManager,  // Gestionnaire de séries
    pub viewport: Viewport,              // Vue visible
    pub interaction: InteractionState,   // État des interactions
}
```

---

## Structures de Tools

### `Tool`

Outils disponibles pour le dessin.

```rust
pub enum Tool {
    Rectangle,        // Outil rectangle
    HorizontalLine,   // Outil ligne horizontale
}
```

### `DrawnRectangle`

Rectangle dessiné sur le graphique.

```rust
pub struct DrawnRectangle {
    pub start_time: i64,   // Timestamp de début (coin gauche)
    pub start_price: f64,  // Prix de début (coin haut ou bas)
    pub end_time: i64,     // Timestamp de fin (coin droit)
    pub end_price: f64,    // Prix de fin (coin opposé)
    pub color: Color,      // Couleur du rectangle (RGBA)
}
```

**Stockage** : En coordonnées de graphique (timestamp, prix) pour rester cohérent avec le zoom/pan.

### `DrawnHorizontalLine`

Ligne horizontale dessinée sur le graphique.

```rust
pub struct DrawnHorizontalLine {
    pub price: f64,   // Prix de la ligne
    pub color: Color, // Couleur de la ligne (RGBA)
}
```

### `DrawingState`

État de dessin en cours.

```rust
pub struct DrawingState {
    pub is_drawing: bool,                        // Indique si un dessin est en cours
    pub start_screen_point: Option<(f32, f32)>,  // Point de départ en pixels
    pub start_time: Option<i64>,                 // Timestamp de départ
    pub start_price: Option<f64>,                // Prix de départ
    pub current_screen_point: Option<(f32, f32)>, // Point actuel en pixels
}
```

### `EditState`

État d'édition d'un rectangle.

```rust
pub struct EditState {
    pub selected_index: Option<usize>,        // Index du rectangle sélectionné
    pub edit_mode: Option<EditMode>,          // Mode d'édition
    pub is_editing: bool,                     // Indique si une édition est en cours
    pub start_time: Option<i64>,              // Timestamp de départ de l'édition
    pub start_price: Option<f64>,             // Prix de départ de l'édition
    pub original_rect: Option<DrawnRectangle>, // Rectangle original (pour undo)
}
```

### `HLineEditState`

État d'édition d'une ligne horizontale.

```rust
pub struct HLineEditState {
    pub selected_index: Option<usize>,           // Index de la ligne sélectionnée
    pub is_editing: bool,                        // Indique si une édition est en cours
    pub start_price: Option<f64>,                // Prix de départ de l'édition
    pub original_line: Option<DrawnHorizontalLine>, // Ligne originale (pour undo)
}
```

### `History`

Gestionnaire d'historique pour undo/redo.

```rust
pub struct History {
    undo_stack: Vec<Action>,  // Pile des actions à annuler
    redo_stack: Vec<Action>,  // Pile des actions à rétablir
}
```

**Limite** : 50 actions maximum dans `undo_stack`.

### `Action`

Action enregistrée dans l'historique.

```rust
pub enum Action {
    CreateRectangle { rect: DrawnRectangle },
    DeleteRectangle { index: usize, rect: DrawnRectangle },
    ModifyRectangle { index: usize, old_rect: DrawnRectangle, new_rect: DrawnRectangle },
    CreateHLine { line: DrawnHorizontalLine },
    DeleteHLine { index: usize, line: DrawnHorizontalLine },
    ModifyHLine { index: usize, old_line: DrawnHorizontalLine, new_line: DrawnHorizontalLine },
}
```

### `ToolsState`

État principal des outils.

```rust
pub struct ToolsState {
    pub rectangles: Vec<DrawnRectangle>,           // Rectangles dessinés
    pub horizontal_lines: Vec<DrawnHorizontalLine>, // Lignes horizontales dessinées
    pub drawing: DrawingState,                      // État de dessin en cours
    pub editing: EditState,                         // État d'édition de rectangle
    pub hline_editing: HLineEditState,             // État d'édition de ligne
    pub history: History,                           // Historique undo/redo
    pub selected_tool: Option<Tool>,                // Outil sélectionné
}
```

**Persistance** : Sauvegardé dans `drawings.json` (JSON).

---

## Structures de Settings

### `ChartStyle`

Style personnalisable du graphique.

```rust
pub struct ChartStyle {
    pub background_color: SerializableColor,    // Couleur de fond
    pub bullish_color: SerializableColor,       // Couleur bougie haussière
    pub bearish_color: SerializableColor,       // Couleur bougie baissière
    pub wick_color: SerializableColor,         // Couleur des mèches
    pub grid_color: SerializableColor,          // Couleur de la grille
    pub current_price_color: SerializableColor, // Couleur ligne prix courant
    pub crosshair_color: SerializableColor,     // Couleur du crosshair
    pub text_color: SerializableColor,          // Couleur du texte
}
```

**Persistance** : Sauvegardé dans `chart_style.json` (JSON).

### `SerializableColor`

Couleur sérialisable (wrapper autour de `iced::Color`).

```rust
pub struct SerializableColor {
    pub r: f32,  // Rouge (0.0 - 1.0)
    pub g: f32,  // Vert (0.0 - 1.0)
    pub b: f32,  // Bleu (0.0 - 1.0)
    pub a: f32,  // Alpha (0.0 - 1.0)
}
```

**Conversions** :
- `from_iced(color: Color) -> Self`
- `to_iced(self) -> Color`

### `SettingsState`

État du dialog settings.

```rust
pub struct SettingsState {
    pub is_open: bool,  // Le dialog est-il ouvert
}
```

---

## Diagramme de relations

```
ChartState
    │
    ├─► SeriesManager
    │       │
    │       ├─► HashMap<SeriesId, SeriesData>
    │       │       │
    │       │       └─► SeriesData
    │       │               ├─► SeriesId
    │       │               ├─► TimeSeries
    │       │               │       ├─► Vec<Candle>
    │       │               │       ├─► PriceRangeCache
    │       │               │       └─► TimeRangeCache
    │       │               ├─► symbol: String
    │       │               ├─► interval: String
    │       │               └─► color: Option<Color>
    │       │
    │       └─► Vec<SeriesId> (active_series)
    │
    ├─► Viewport
    │       ├─► PriceScale
    │       └─► TimeScale
    │
    └─► InteractionState

ToolsState
    ├─► Vec<DrawnRectangle>
    ├─► Vec<DrawnHorizontalLine>
    ├─► DrawingState
    ├─► EditState
    ├─► HLineEditState
    ├─► History
    │       ├─► Vec<Action> (undo_stack)
    │       └─► Vec<Action> (redo_stack)
    └─► Option<Tool>

ChartStyle
    └─► SerializableColor (×8)
```

---

## Formats de sérialisation

### JSON - Drawings

```json
{
  "rectangles": [
    {
      "start_time": 1609459200,
      "start_price": 29374.15,
      "end_time": 1609462800,
      "end_price": 29380.00,
      "color": { "r": 0.2, "g": 0.6, "b": 1.0, "a": 0.3 }
    }
  ],
  "horizontal_lines": [
    {
      "price": 29375.00,
      "color": { "r": 1.0, "g": 0.0, "b": 0.0, "a": 0.8 }
    }
  ]
}
```

### JSON - Chart Style

```json
{
  "background_color": { "r": 0.06, "g": 0.06, "b": 0.08, "a": 1.0 },
  "bullish_color": { "r": 0.0, "g": 0.8, "b": 0.0, "a": 1.0 },
  "bearish_color": { "r": 0.8, "g": 0.0, "b": 0.0, "a": 1.0 },
  ...
}
```

### JSON - Data

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

