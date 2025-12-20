# API Simplifiée

## Vue d'ensemble

L'API simplifiée permet de créer une application de graphique en **10-15 lignes de code** au lieu de plusieurs centaines.

## Utilisation ultra-simple

### Exemple minimal (3 lignes)

```rust
use candlechart::simple_app;

fn main() -> iced::Result {
    simple_app("data", 1200.0, 800.0)
}
```

C'est tout ! L'application :
- ✅ Charge automatiquement toutes les séries depuis `data/`
- ✅ Crée tous les états nécessaires
- ✅ Configure l'interface utilisateur
- ✅ Gère tous les messages
- ✅ Lance l'application

### Exemple avec personnalisation (10-15 lignes)

```rust
use candlechart::SimpleChartApp;

fn main() -> iced::Result {
    let app = SimpleChartApp::new("data", 1200.0, 800.0)
        .with_title("Mon Graphique")
        .with_style_file("chart_style.json")
        .with_drawings_file("drawings.json");
    
    // Lancer l'application
    candlechart::simple_app_from_app(app)
}
```

## API Simplifiée vs API Avancée

### Quand utiliser l'API Simplifiée ?

✅ **Utilisez l'API simplifiée si** :
- Vous voulez un graphique fonctionnel rapidement
- Vous n'avez pas besoin de personnalisation complexe
- Vous chargez des données depuis un dossier JSON
- Vous voulez le comportement par défaut

### Quand utiliser l'API Avancée ?

✅ **Utilisez l'API avancée si** :
- Vous avez besoin de personnaliser l'interface
- Vous voulez gérer des messages spécifiques
- Vous avez besoin de plusieurs fenêtres
- Vous voulez intégrer le graphique dans une application plus large

## Fonctions disponibles

### `simple_app(data_path, width, height)`

Lance une application complète avec les paramètres par défaut.

**Paramètres** :
- `data_path` : Chemin vers le dossier contenant les fichiers JSON
- `width` : Largeur de la fenêtre en pixels
- `height` : Hauteur de la fenêtre en pixels

**Retour** : `iced::Result`

### `SimpleChartApp::new(data_path, width, height)`

Crée une nouvelle application simplifiée (sans la lancer).

**Méthodes de personnalisation** :
- `.with_title(title)` : Définit un titre personnalisé
- `.with_style_file(path)` : Charge un style depuis un fichier
- `.with_drawings_file(path)` : Charge des dessins depuis un fichier
- `.chart_state_mut()` : Accès mutable à l'état pour personnalisation avancée
- `.tools_state_mut()` : Accès mutable aux outils
- `.style_mut()` : Accès mutable au style

## Exemples complets

### Exemple 1 : Minimal

```rust
use candlechart::simple_app;

fn main() -> iced::Result {
    simple_app("data", 1200.0, 800.0)
}
```

### Exemple 2 : Avec titre personnalisé

```rust
use candlechart::SimpleChartApp;

fn main() -> iced::Result {
    let app = SimpleChartApp::new("data", 1200.0, 800.0)
        .with_title("Analyse BTCUSDT");
    
    // Note: Pour l'instant, utilisez simple_app() directement
    // Une fonction simple_app_from_app() sera ajoutée si nécessaire
    candlechart::simple_app("data", 1200.0, 800.0)
}
```

### Exemple 3 : Personnalisation avancée

```rust
use candlechart::{SimpleChartApp, ChartStyle};

fn main() -> iced::Result {
    let mut app = SimpleChartApp::new("data", 1200.0, 800.0);
    
    // Personnaliser le style
    app.style_mut().background_color = /* ... */;
    
    // Personnaliser l'état
    app.chart_state_mut().zoom(0.5);
    
    // Lancer (via simple_app pour l'instant)
    candlechart::simple_app("data", 1200.0, 800.0)
}
```

## Migration depuis l'API Avancée

Si vous avez du code utilisant l'API avancée, vous pouvez le simplifier :

### Avant (API Avancée - ~250 lignes)

```rust
struct MyApp {
    chart_state: ChartState,
    tools_state: ToolsState,
    // ... beaucoup de code
}

impl MyApp {
    fn new() -> (Self, Task<Message>) { /* ... */ }
    fn update(&mut self, message: Message) -> Task<Message> { /* ... */ }
    fn view(&self, window_id: window::Id) -> Element<'_, Message> { /* ... */ }
    // ... beaucoup de méthodes
}

fn main() -> iced::Result {
    iced::daemon(MyApp::new, MyApp::update, MyApp::view)
        .title(MyApp::title)
        .theme(MyApp::theme)
        .subscription(MyApp::subscription)
        .run()
}
```

### Après (API Simplifiée - 3 lignes)

```rust
use candlechart::simple_app;

fn main() -> iced::Result {
    simple_app("data", 1200.0, 800.0)
}
```

## Limitations de l'API Simplifiée

L'API simplifiée ne gère pas (pour l'instant) :
- Fenêtre de settings personnalisée
- Messages de dessin/édition complexes
- Intégration dans une application multi-fenêtres
- Personnalisation complète de l'UI

Pour ces cas, utilisez l'API avancée.

## Prochaines améliorations

- [ ] Fonction `simple_app_from_app()` pour utiliser un `SimpleChartApp` personnalisé
- [ ] Support des callbacks pour personnaliser les messages
- [ ] Configuration via un fichier de config
- [ ] Support de thèmes personnalisés

