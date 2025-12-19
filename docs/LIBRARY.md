# Utilisation comme Librairie

## ✅ Statut

Le code **peut être utilisé comme crate/librairie**. Le projet est maintenant configuré pour fonctionner à la fois comme :
- **Binaire** : Application exécutable (`cargo run`)
- **Librairie** : Crate réutilisable (`cargo build --lib`)

## Configuration

### Fichiers créés/modifiés

1. **`src/lib.rs`** : Point d'entrée de la librairie
   - Expose le module `finance_chart`
   - Ré-exporte les types et fonctions publiques
   - Documentation avec exemples

2. **`Cargo.toml`** : Configuration mise à jour
   - Section `[lib]` : Définit la librairie
   - Section `[[bin]]` : Définit le binaire
   - Métadonnées du package (description, licence, etc.)

## Utilisation comme dépendance

### Dans un autre projet Rust

#### 1. Ajouter la dépendance

**Si publié sur crates.io** :
```toml
[dependencies]
candlechart = "0.1.0"
```

**Si local (chemin relatif)** :
```toml
[dependencies]
candlechart = { path = "../CandleChart" }
```

**Si depuis Git** :
```toml
[dependencies]
candlechart = { git = "https://github.com/yourusername/CandleChart" }
```

#### 2. Utiliser dans votre code

```rust
use candlechart::{
    ChartState, chart, load_from_json,
    ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    ChartStyle, SettingsState,
    ChartMessage,
    Candle, TimeSeries, SeriesManager,
    PriceScale, TimeScale, Viewport,
};
use iced::Element;

fn main() -> iced::Result {
    // Charger des données
    let series = load_from_json("data/BTCUSDT_1h.json")?;
    
    // Créer l'état du graphique
    let mut chart_state = ChartState::new(1200.0, 800.0);
    chart_state.add_series(series);
    
    // Créer les widgets
    let tools_state = ToolsState::default();
    let settings_state = SettingsState::default();
    let chart_style = ChartStyle::default();
    
    // Créer le widget graphique
    let chart_widget: Element<ChartMessage> = chart(
        &chart_state,
        &tools_state,
        &settings_state,
        &chart_style,
    );
    
    // Utiliser dans votre application Iced
    // ...
}
```

## API Publique

### Types principaux

#### Core
- `Candle` : Structure OHLC
- `TimeSeries` : Collection de bougies avec cache
- `SeriesId` : Identifiant de série
- `SeriesData` : Série avec métadonnées
- `SeriesManager` : Gestionnaire de plusieurs séries

#### Scale
- `PriceScale` : Conversion prix → coordonnées Y
- `TimeScale` : Conversion timestamp → coordonnées X

#### Viewport
- `Viewport` : Gestion de la vue visible

#### State
- `ChartState` : État global du graphique

#### Tools
- `ToolsState` : État des outils de dessin

#### Settings
- `ChartStyle` : Style personnalisable
- `SettingsState` : État des settings

### Fonctions principales

#### Chargement de données
- `load_from_json(path)` : Charge une série depuis JSON
- `load_all_from_directory(dir)` : Charge toutes les séries d'un dossier

#### Widgets
- `chart(...)` : Widget canvas principal
- `x_axis(...)` : Widget axe X
- `y_axis(...)` : Widget axe Y
- `tools_panel(...)` : Panel d'outils
- `series_select_box(...)` : Select box de séries

### Messages

- `ChartMessage` : Messages du canvas principal
- `YAxisMessage` : Messages de l'axe Y
- `XAxisMessage` : Messages de l'axe X
- `ToolsPanelMessage` : Messages du panel d'outils
- `SeriesPanelMessage` : Messages de sélection de série

### Constantes

- `X_AXIS_HEIGHT` : Hauteur de l'axe X
- `Y_AXIS_WIDTH` : Largeur de l'axe Y
- `TOOLS_PANEL_WIDTH` : Largeur du panel d'outils

## Exemple complet

```rust
use candlechart::{
    ChartState, chart, load_all_from_directory,
    ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    ChartStyle, SettingsState,
    ChartMessage,
};
use iced::{
    widget::{column, row, container, text},
    Element, Length, Color,
};

struct MyApp {
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
}

impl MyApp {
    fn new() -> Self {
        let mut chart_state = ChartState::new(1200.0, 800.0);
        
        // Charger toutes les séries
        if let Ok(series_list) = load_all_from_directory("data") {
            for series in series_list {
                chart_state.add_series(series);
            }
        }
        
        Self {
            chart_state,
            tools_state: ToolsState::default(),
            settings_state: SettingsState::default(),
            chart_style: ChartStyle::default(),
        }
    }
    
    fn view(&self) -> Element<ChartMessage> {
        column![
            text("Mon Application de Trading")
                .size(24)
                .color(Color::WHITE),
            row![
                tools_panel(&self.tools_state),
                chart(
                    &self.chart_state,
                    &self.tools_state,
                    &self.settings_state,
                    &self.chart_style,
                ),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        ]
        .into()
    }
}
```

## Utilisation avancée

### Accès aux modules internes

Si vous avez besoin d'accéder aux modules internes :

```rust
use candlechart::finance_chart::{
    core::Candle,
    render::render_candlesticks,
    // ...
};
```

### Extension de fonctionnalités

Vous pouvez étendre la librairie en :

1. **Créant vos propres widgets** utilisant les types publics
2. **Implémentant des trait personnalisés** pour les types
3. **Composant plusieurs widgets** ensemble

## Compilation

### Compiler la librairie uniquement

```bash
cargo build --lib
```

### Compiler le binaire uniquement

```bash
cargo build --bin CandleChart
```

### Compiler les deux

```bash
cargo build
```

### Documentation de la librairie

```bash
# Générer la documentation
cargo doc --lib --open

# Documentation avec dépendances privées
cargo doc --lib --document-private-items
```

## Publication sur crates.io

Pour publier la librairie sur crates.io :

1. **Mettre à jour `Cargo.toml`** :
   - Ajouter `license`
   - Ajouter `repository`
   - Ajouter `description`
   - Vérifier `version`

2. **Créer un compte sur crates.io**

3. **Publier** :
   ```bash
   cargo publish
   ```

## Limitations actuelles

1. **Dépendance Iced** : La librairie dépend d'Iced 0.14, ce qui peut limiter la compatibilité avec d'autres versions
2. **API publique** : Certains types internes ne sont pas encore exportés publiquement
3. **Documentation** : Certaines fonctions pourraient bénéficier de plus d'exemples

## Améliorations possibles

1. **Features flags** : Permettre d'activer/désactiver certaines fonctionnalités
   ```toml
   [features]
   default = ["render", "tools"]
   render = []
   tools = []
   ```

2. **Traits** : Créer des traits pour permettre l'extension
   ```rust
   pub trait DataSource {
       fn load_candles(&self) -> Result<Vec<Candle>>;
   }
   ```

3. **Export conditionnel** : Exporter certains modules seulement si activés

## Conclusion

Le code est **prêt à être utilisé comme librairie**. La structure modulaire et les exports publics permettent une utilisation facile dans d'autres projets Rust.

