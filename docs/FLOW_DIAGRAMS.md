# Diagrammes de Flux

## Table des matières

1. [Flux de chargement](#flux-de-chargement)
2. [Flux d'interaction](#flux-dinteraction)
3. [Flux de rendu](#flux-de-rendu)
4. [Flux de dessin](#flux-de-dessin)
5. [Flux d'édition](#flux-dédition)

---

## Flux de chargement

### Chargement initial de l'application

```
main()
    │
    ▼
ChartApp::new()
    │
    ├─► ChartState::new(1200.0, 800.0)
    │       │
    │       ├─► SeriesManager::new()
    │       ├─► Viewport::new()
    │       └─► InteractionState::default()
    │
    ├─► load_all_from_directory("data")
    │       │
    │       ├─► Pour chaque fichier .json :
    │       │       │
    │       │       ├─► load_from_json(path)
    │       │       │       │
    │       │       │       ├─► Ouvrir fichier
    │       │       │       ├─► Parser JSON
    │       │       │       ├─► Convertir timestamps (ms → s)
    │       │       │       ├─► Créer TimeSeries
    │       │       │       └─► Créer SeriesData
    │       │       │
    │       │       └─► ChartState::add_series()
    │       │               │
    │       │               ├─► SeriesManager::add_series()
    │       │               │       │
    │       │               │       └─► Active automatiquement la première série
    │       │               │
    │       │               └─► update_viewport_from_series()
    │       │                       │
    │       │                       └─► Viewport::focus_on_recent()
    │       │                               │
    │       │                               ├─► Calculer plage temporelle
    │       │                               ├─► Calculer plage de prix (visible)
    │       │                               └─► Mettre à jour les échelles
    │       │
    │       └─► Retourner Vec<SeriesData>
    │
    ├─► ToolsState::load_from_file("drawings.json")
    │       │
    │       └─► Charger rectangles et lignes horizontales
    │
    ├─► ChartStyle::load_from_file("chart_style.json")
    │       │
    │       └─► Charger les couleurs (ou utiliser défaut)
    │
    └─► window::open() → Créer fenêtre principale
```

---

## Flux d'interaction

### Pan (Déplacement)

```
Mouse Event: ButtonPressed(Left)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_press(position)
    │       │
    │       └─► ChartMessage::StartPan { position }
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ChartState::start_pan(position)
            │
            └─► InteractionState::start_pan()
                    │
                    └─► Enregistrer position de départ

Mouse Event: CursorMoved
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_move(position)
    │       │
    │       └─► ChartMessage::UpdatePan { position }
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ChartState::update_pan(position)
            │
            ├─► InteractionState::update_pan()
            │       │
            │       └─► Calculer delta (x, y)
            │
            ├─► Viewport::pan_horizontal(delta_x)
            │       │
            │       └─► Mettre à jour TimeScale
            │
            └─► Viewport::pan_vertical(delta_y)
                    │
                    └─► Mettre à jour PriceScale

Mouse Event: ButtonReleased(Left)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_release()
    │       │
    │       └─► ChartMessage::EndPan
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ChartState::end_pan()
            │
            └─► InteractionState::end_pan()
```

### Zoom

```
Mouse Event: WheelScrolled
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_scroll(delta)
    │       │
    │       ├─► Si ALT pressé :
    │       │       └─► ChartMessage::ZoomVertical { factor }
    │       │
    │       ├─► Si CTRL pressé :
    │       │       └─► ChartMessage::ZoomBoth { factor }
    │       │
    │       └─► Sinon :
    │               └─► ChartMessage::ZoomHorizontal { factor }
    │
    ▼
main.rs::handle_chart_message()
    │
    ├─► ChartState::zoom(factor)
    │       │
    │       └─► Viewport::zoom()
    │               │
    │               ├─► Calculer nouveau time_range
    │               ├─► Appliquer limites min/max
    │               └─► Mettre à jour TimeScale
    │
    ├─► ChartState::zoom_vertical(factor)
    │       │
    │       └─► Viewport::zoom_vertical()
    │               │
    │               ├─► Calculer nouveau price_range
    │               ├─► Appliquer limites min/max
    │               └─► Mettre à jour PriceScale
    │
    └─► ChartState::zoom_both(factor)
            │
            └─► Viewport::zoom_both()
                    │
                    ├─► zoom_horizontal()
                    └─► zoom_vertical()
```

---

## Flux de rendu

### Cycle de rendu complet

```
Iced Framework
    │
    ▼
ChartProgram::draw()
    │
    ├─► Créer Frame
    │
    ├─► Rendre fond
    │       │
    │       └─► frame.fill_rectangle() avec background_color
    │
    ├─► Rendre grille
    │       │
    │       ├─► render_grid()
    │       │       │
    │       │       ├─► Calculer pas "nice" pour prix
    │       │       ├─► Dessiner lignes horizontales
    │       │       ├─► Calculer pas "nice" pour temps
    │       │       └─► Dessiner lignes verticales
    │
    ├─► Rendre bougies
    │       │
    │       ├─► ChartState::visible_candles()
    │       │       │
    │       │       ├─► Récupérer plage temporelle du viewport
    │       │       └─► SeriesManager::visible_candles()
    │       │               │
    │       │               └─► Pour chaque série active :
    │       │                       └─► TimeSeries::visible_candles()
    │       │                               │
    │       │                               └─► Recherche binaire
    │       │
    │       └─► render_candlesticks()
    │               │
    │               ├─► Pour chaque bougie :
    │               │       │
    │               │       ├─► Convertir timestamp → X (TimeScale)
    │               │       ├─► Convertir prix → Y (PriceScale)
    │               │       ├─► Calculer largeur adaptative
    │               │       └─► Dessiner bougie (corps + mèches)
    │
    ├─► Rendre ligne de prix courant
    │       │
    │       ├─► ChartState::last_candle()
    │       │       │
    │       │       └─► Récupérer dernière bougie de série active
    │       │
    │       └─► render_current_price_line()
    │               │
    │               └─► Dessiner ligne horizontale au prix de clôture
    │
    ├─► Rendre dessins
    │       │
    │       ├─► Pour chaque rectangle :
    │       │       │
    │       │       ├─► Convertir coordonnées (timestamp, prix) → pixels
    │       │       └─► draw_rectangle()
    │       │
    │       └─► Pour chaque ligne horizontale :
    │               │
    │               ├─► Convertir prix → Y
    │               └─► draw_horizontal_line()
    │
    ├─► Rendre crosshair (si souris présente)
    │       │
    │       ├─► render_crosshair()
    │       │       │
    │       │       ├─► Dessiner lignes verticale et horizontale
    │       │       ├─► Convertir position → prix/temps
    │       │       └─► Afficher labels prix/temps
    │
    └─► Rendre tooltip (si SHIFT maintenu)
            │
            ├─► find_candle_at_position()
            │       │
            │       └─► Trouver bougie sous la souris
            │
            └─► render_tooltip()
                    │
                    └─► Afficher valeurs OHLC
```

---

## Flux de dessin

### Dessin d'un rectangle

```
Mouse Event: ButtonPressed(Left) + Tool::Rectangle sélectionné
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_press(position)
    │       │
    │       ├─► Convertir position → (time, price)
    │       └─► ChartMessage::StartDrawingRectangle
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ToolsState::drawing.start()
            │
            └─► Enregistrer point de départ

Mouse Event: CursorMoved (pendant le dessin)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_move(position)
    │       │
    │       └─► ChartMessage::UpdateDrawing
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ToolsState::drawing.update()
            │
            └─► Mettre à jour point actuel
                    │
                    └─► Re-render avec aperçu

Mouse Event: ButtonReleased(Left)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_release()
    │       │
    │       ├─► Convertir position → (time, price)
    │       └─► ChartMessage::FinishDrawingRectangle
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ToolsState::drawing.finish()
            │
            ├─► Créer DrawnRectangle
            ├─► History::record(CreateRectangle)
            ├─► Ajouter à rectangles
            ├─► Sélectionner le nouveau rectangle
            └─► Désélectionner l'outil
```

### Dessin d'une ligne horizontale

```
Mouse Event: ButtonPressed(Left) + Tool::HorizontalLine sélectionné
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_press(position)
    │       │
    │       ├─► Convertir Y → price
    │       └─► ChartMessage::StartDrawingHLine
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ToolsState::drawing.start()
            │
            └─► Enregistrer prix de départ

Mouse Event: ButtonReleased(Left)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_release()
    │       │
    │       └─► ChartMessage::FinishDrawingHLine
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► ToolsState::drawing.finish_hline()
            │
            ├─► Créer DrawnHorizontalLine
            ├─► History::record(CreateHLine)
            ├─► Ajouter à horizontal_lines
            ├─► Sélectionner la nouvelle ligne
            └─► Désélectionner l'outil
```

---

## Flux d'édition

### Édition d'un rectangle

```
Mouse Event: ButtonPressed(Left) sur rectangle sélectionné
    │
    ▼
ChartProgram::update()
    │
    ├─► hit_test_rectangles()
    │       │
    │       └─► Détecter zone cliquée (coin, bord, centre)
    │
    ├─► Déterminer EditMode
    │       │
    │       ├─► Centre → Move
    │       ├─► Coin → ResizeTopLeft, etc.
    │       └─► Bord → ResizeTop, etc.
    │
    └─► ChartMessage::StartRectangleEdit
            │
            └─► Inclure index, mode, time, price

main.rs::handle_chart_message()
    │
    └─► ToolsState::editing.start()
            │
            ├─► Enregistrer rectangle original (pour undo)
            ├─► Enregistrer mode d'édition
            └─► Enregistrer point de départ

Mouse Event: CursorMoved (pendant l'édition)
    │
    ▼
ChartProgram::update()
    │
    ├─► handle_mouse_move(position)
    │       │
    │       ├─► Convertir position → (time, price)
    │       └─► ChartMessage::UpdateRectangleEdit
    │
    ▼
main.rs::handle_chart_message()
    │
    └─► apply_edit_update()
            │
            ├─► Calculer delta depuis point de départ
            ├─► Appliquer selon EditMode
            │       │
            │       ├─► Move : Déplacer rectangle
            │       └─► Resize* : Redimensionner depuis zone
            │
            └─► Mettre à jour rectangle en place

Mouse Event: ButtonReleased(Left)
    │
    ▼
ChartProgram::update()
    │
    └─► ChartMessage::FinishRectangleEdit

main.rs::handle_chart_message()
    │
    └─► Vérifier si modification
            │
            ├─► Si modifié :
            │       └─► History::record(ModifyRectangle)
            │
            └─► ToolsState::editing.finish()
```

### Édition d'une ligne horizontale

```
Mouse Event: ButtonPressed(Left) sur ligne sélectionnée
    │
    ▼
ChartProgram::update()
    │
    ├─► hit_test_hline()
    │       │
    │       └─► Détecter si souris sur ligne
    │
    └─► ChartMessage::StartHLineEdit
            │
            └─► Inclure index, price

main.rs::handle_chart_message()
    │
    └─► ToolsState::hline_editing.start()
            │
            ├─► Enregistrer ligne originale
            └─► Enregistrer prix de départ

Mouse Event: CursorMoved (pendant l'édition)
    │
    ▼
ChartProgram::update()
    │
    ├─► Convertir Y → price
    └─► ChartMessage::UpdateHLineEdit

main.rs::handle_chart_message()
    │
    └─► Calculer delta depuis prix de départ
            │
            └─► Mettre à jour prix de la ligne

Mouse Event: ButtonReleased(Left)
    │
    ▼
ChartProgram::update()
    │
    └─► ChartMessage::FinishHLineEdit

main.rs::handle_chart_message()
    │
    └─► Vérifier si modification
            │
            ├─► Si modifié :
            │       └─► History::record(ModifyHLine)
            │
            └─► ToolsState::hline_editing.finish()
```

---

## Flux de changement de série

```
PickList Selection
    │
    ▼
SeriesPanelMessage::SelectSeriesByName { series_name }
    │
    ▼
main.rs::update()
    │
    ├─► Trouver SeriesId depuis series_name
    │       │
    │       └─► SeriesManager::all_series()
    │               │
    │               └─► find(|s| s.full_name() == series_name)
    │
    ├─► SeriesManager::activate_only_series(series_id)
    │       │
    │       ├─► active_series.clear()
    │       └─► active_series.push(series_id)
    │
    └─► ChartState::update_viewport_from_series()
            │
            └─► Viewport::focus_on_recent()
                    │
                    ├─► Récupérer série active
                    ├─► Calculer plage temporelle (N dernières bougies)
                    ├─► Calculer plage de prix (bougies visibles)
                    │       │
                    │       └─► Utiliser cache si disponible
                    │
                    └─► Mettre à jour PriceScale et TimeScale
                            │
                            └─► Re-render automatique
```

---

## Diagramme de séquence - Interaction complète

```
Utilisateur    ChartProgram    main.rs        ChartState      Viewport
    │              │              │              │              │
    │─Clic───────►│              │              │              │
    │              │─Message────►│              │              │
    │              │              │─update()────►│              │
    │              │              │              │─pan()───────►│
    │              │              │              │              │─update()
    │              │              │              │◄─────────────│
    │              │              │◄─────────────│              │
    │              │◄─────────────│              │              │
    │◄─Re-render───│              │              │              │
    │              │              │              │              │
```

---

## Optimisations

### Cache des plages de prix

```
TimeSeries::price_range()
    │
    ├─► Vérifier cache global
    │       │
    │       └─► Si présent : Retourner immédiatement
    │
    └─► Si absent :
            │
            ├─► Calculer en itérant sur toutes les bougies
            └─► Mettre en cache
                    │
                    └─► Retourner résultat
```

### Recherche binaire pour visible_candles

```
TimeSeries::visible_candles(time_range)
    │
    ├─► binary_search_by_key(time_range.start)
    │       │
    │       └─► Trouver index de début (O(log n))
    │
    ├─► binary_search_by_key(time_range.end)
    │       │
    │       └─► Trouver index de fin (O(log n))
    │
    └─► Retourner slice [start..end] (O(1))
```

**Complexité totale** : O(log n) au lieu de O(n) avec recherche linéaire.

