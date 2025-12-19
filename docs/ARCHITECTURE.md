# Architecture CandleChart

## Vue d'ensemble

CandleChart suit une architecture modulaire basée sur le pattern **Elm Architecture**, garantissant une séparation claire des responsabilités et une maintenabilité élevée.

## Pattern Elm Architecture

```
┌─────────────┐
│   Events    │ ────┐
│  (Souris,   │     │
│  Clavier)   │     │
└─────────────┘     │
                    │
                    ▼
            ┌───────────────┐
            │   Messages    │
            │  (Typés)      │
            └───────────────┘
                    │
                    ▼
            ┌───────────────┐
            │    Update     │
            │ (Transforme   │
            │   l'état)     │
            └───────────────┘
                    │
                    ▼
            ┌───────────────┐
            │     State     │
            │  (Immuable)   │
            └───────────────┘
                    │
                    ▼
            ┌───────────────┐
            │     View      │
            │  (Rendu)      │
            └───────────────┘
```

## Architecture modulaire

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              ChartApp (Application)                   │  │
│  │  • Gère les fenêtres                                  │  │
│  │  • Orchestre les messages                             │  │
│  │  • Coordonne les différents états                     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ utilise
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    finance_chart/                           │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    core/     │  │    scale/    │  │  viewport/   │     │
│  │              │  │              │  │              │     │
│  │ • Candle     │  │ • PriceScale │  │ • Viewport   │     │
│  │ • TimeSeries │  │ • TimeScale  │  │ • Zoom/Pan   │     │
│  │ • SeriesMgr  │  │              │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   render/    │  │ interaction/ │  │   state/     │     │
│  │              │  │              │  │              │     │
│  │ • Candles    │  │ • Events     │  │ • ChartState │     │
│  │ • Grid       │  │ • Rectangle  │  │              │     │
│  │ • Crosshair  │  │   Editing    │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   widget.rs  │  │data_loader.rs│  │  settings.rs │     │
│  │              │  │              │  │              │     │
│  │ • Canvas     │  │ • JSON Load  │  │ • ChartStyle │     │
│  │ • Program    │  │              │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Flux de données

### Chargement initial

```
main.rs::new()
    │
    ├─► load_all_from_directory("data")
    │       │
    │       ├─► load_from_json() pour chaque fichier
    │       │       │
    │       │       └─► SeriesData::new()
    │       │
    │       └─► ChartState::add_series()
    │               │
    │               ├─► SeriesManager::add_series()
    │               │
    │               └─► update_viewport_from_series()
    │                       │
    │                       └─► Viewport::focus_on_recent()
    │
    ├─► ToolsState::load_from_file("drawings.json")
    │
    └─► ChartStyle::load_from_file("chart_style.json")
```

### Interaction utilisateur

```
Event (Souris/Clavier)
    │
    ▼
ChartProgram::update()
    │
    ├─► Détection du type d'événement
    │
    ├─► Création d'un ChartMessage
    │
    ▼
main.rs::handle_chart_message()
    │
    ├─► Modification de ChartState
    │   │
    │   ├─► Pan : viewport.pan_horizontal/vertical()
    │   ├─► Zoom : viewport.zoom() / zoom_vertical()
    │   ├─► Drawing : tools_state.drawing.start/update/finish()
    │   └─► Editing : tools_state.editing.start/update/finish()
    │
    └─► Re-render automatique (Iced)
```

### Changement de série

```
PickList Selection
    │
    ▼
SeriesPanelMessage::SelectSeriesByName
    │
    ▼
main.rs::update()
    │
    ├─► Trouver SeriesId depuis le nom
    │
    ├─► SeriesManager::activate_only_series()
    │   │
    │   └─► Désactive toutes les séries
    │       Active uniquement la série sélectionnée
    │
    └─► ChartState::update_viewport_from_series()
            │
            └─► Viewport::focus_on_recent()
                    │
                    └─► Réinitialise le zoom
```

## Séparation des responsabilités

### Core (Modèles de données)

**Responsabilité** : Structures de données financières pures

- `Candle` : Structure OHLC + timestamp
- `TimeSeries` : Collection ordonnée de bougies
- `SeriesManager` : Gestion de plusieurs séries
- `Cache` : Optimisation des calculs de plages

**Pas de dépendances** vers Iced ou le rendu

### Scale (Conversion coordonnées)

**Responsabilité** : Conversion données → pixels

- `PriceScale` : Prix (f64) → Coordonnée Y (f32)
- `TimeScale` : Timestamp (i64) → Coordonnée X (f32)

**Indépendant** du rendu et de l'UI

### Viewport (Gestion de la vue)

**Responsabilité** : Définit quelle portion des données est visible

- Zoom horizontal/vertical
- Pan (déplacement)
- Focus sur les données récentes
- Limites min/max pour éviter les extrêmes

**Utilise** Scale pour les conversions

### Render (Rendu)

**Responsabilité** : Dessin des éléments graphiques

- Bougies (candlesticks)
- Grille
- Crosshair
- Tooltip
- Rectangles et lignes horizontales

**Fonctions pures** : Reçoivent des données, dessinent sur un Frame

### Interaction (Événements)

**Responsabilité** : Gestion des interactions utilisateur

- Détection des zones cliquables
- Gestion de l'édition de rectangles
- États de dessin

**Séparé** de la logique métier

### State (État)

**Responsabilité** : État global du graphique

- `ChartState` : Combine SeriesManager, Viewport, InteractionState
- Centralise l'état pour faciliter la gestion

### Widget (UI)

**Responsabilité** : Widget canvas Iced

- Implémente `Program<ChartMessage>`
- Gère les événements souris/clavier
- Appelle les fonctions de rendu
- Émet des messages pour les mutations

## Système de cache

```
TimeSeries
    │
    ├─► PriceRangeCache
    │   ├─► Cache global (toute la série)
    │   └─► Cache par plage temporelle (HashMap limitée à 100)
    │
    └─► TimeRangeCache
        └─► Cache global (min/max timestamps)

Invalidation automatique lors de push()
```

## Gestion des messages

```
Message (enum principal)
    │
    ├─► Chart(ChartMessage)
    │   ├─► Navigation (Pan, Zoom)
    │   ├─► Drawing (Rectangle, HLine)
    │   ├─► Editing (Move, Resize)
    │   ├─► History (Undo, Redo)
    │   └─► Persistence (Save, Load)
    │
    ├─► YAxis(YAxisMessage)
    │   └─► ZoomVertical
    │
    ├─► XAxis(XAxisMessage)
    │   └─► ZoomHorizontal
    │
    ├─► ToolsPanel(ToolsPanelMessage)
    │   └─► ToggleTool
    │
    └─► SeriesPanel(SeriesPanelMessage)
        └─► SelectSeriesByName
```

## Avantages de cette architecture

1. **Séparation claire** : Chaque module a une responsabilité unique
2. **Testabilité** : Logique métier isolée, facile à tester
3. **Maintenabilité** : Modifications localisées
4. **Extensibilité** : Ajout de fonctionnalités facilité
5. **Performance** : Cache pour optimiser les calculs coûteux
6. **Type safety** : Messages typés, pas de `Any` ou `dyn Trait`

## Points d'attention

1. **Immutabilité** : Les références passées au rendu sont immuables
2. **Messages** : Toutes les mutations passent par des messages
3. **Cache** : Invalidation nécessaire lors des modifications de données
4. **Viewport** : Doit être synchronisé avec les données actives

