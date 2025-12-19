# Documentation CandleChart

Application de visualisation de graphiques financiers (candlesticks) développée en Rust avec le framework Iced.

## 📚 Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Architecture](#architecture)
3. [Modules](#modules)
4. [Guide d'utilisation](#guide-dutilisation)
5. [Référence API](#référence-api)
6. [Structures de données](#structures-de-données)
7. [Diagrammes de flux](#diagrammes-de-flux)

## Vue d'ensemble

CandleChart est une application de visualisation de données financières permettant :
- L'affichage de graphiques en chandeliers (candlesticks)
- La gestion de plusieurs séries temporelles
- Le zoom et le pan interactifs
- Le dessin d'annotations (rectangles, lignes horizontales)
- La personnalisation des couleurs et styles
- La persistance des dessins et styles

### Technologies utilisées

- **Rust** : Langage de programmation
- **Iced 0.14** : Framework GUI cross-platform
- **Serde** : Sérialisation/désérialisation
- **Chrono** : Gestion des dates et temps

### Structure du projet

```
CandleChart/
├── src/
│   ├── main.rs                    # Point d'entrée
│   └── finance_chart/             # Module principal
│       ├── core/                  # Modèles de données
│       ├── scale/                 # Conversion coordonnées
│       ├── viewport/              # Gestion de la vue
│       ├── render/                # Logique de dessin
│       ├── interaction/           # Gestion événements
│       ├── state/                 # État de l'application
│       ├── widget.rs              # Widget canvas principal
│       ├── data_loader.rs         # Chargement JSON
│       └── ...
├── data/                          # Fichiers JSON de données
├── docs/                          # Documentation
└── Cargo.toml                     # Configuration Rust
```

## Architecture

Voir [ARCHITECTURE.md](./ARCHITECTURE.md) pour une description détaillée de l'architecture.

L'application suit le pattern **Elm Architecture** :
- **Messages** : Communication via messages typés
- **State** : État centralisé et immuable
- **View** : Fonctions de rendu pures
- **Update** : Transformations d'état pures

## Modules

Voir [MODULES.md](./MODULES.md) pour la documentation complète de chaque module.

### Modules principaux

- **core** : Structures de données financières (Candle, TimeSeries, SeriesManager)
- **scale** : Conversion prix/temps → coordonnées écran
- **viewport** : Gestion de la vue visible (zoom, pan)
- **render** : Rendu des éléments graphiques
- **interaction** : Gestion des événements utilisateur
- **state** : État global du graphique
- **widget** : Widget canvas principal

## Guide d'utilisation

Voir [USAGE.md](./USAGE.md) pour un guide complet d'utilisation.

### Fonctionnalités principales

1. **Navigation** :
   - Clic gauche + drag : Pan (déplacement)
   - Molette : Zoom horizontal
   - ALT + Molette : Zoom vertical
   - CTRL + Molette : Zoom sur les deux axes

2. **Dessin** :
   - Sélectionner un outil (Rectangle ou Ligne horizontale)
   - Dessiner sur le graphique
   - Éditer les éléments dessinés (déplacement, redimensionnement)

3. **Sélection de série** :
   - Utiliser le select box en haut à droite
   - Le graphique se met à jour automatiquement

4. **Personnalisation** :
   - Cliquer sur l'icône ⚙ pour ouvrir les settings
   - Modifier les couleurs du graphique
   - Les styles sont sauvegardés automatiquement

## Référence API

Voir [API.md](./API.md) pour la référence complète de l'API publique.

## Structures de données

Voir [DATA_STRUCTURES.md](./DATA_STRUCTURES.md) pour la documentation des structures de données.

## Diagrammes de flux

Voir [FLOW_DIAGRAMS.md](./FLOW_DIAGRAMS.md) pour les diagrammes de flux des interactions.

## Installation et compilation

```bash
# Compiler le projet
cargo build

# Exécuter
cargo run

# Tests
cargo test

# Documentation
cargo doc --open
```

## Format des données

Les données doivent être au format JSON avec la structure suivante :

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

Les fichiers JSON doivent être placés dans le dossier `data/`.

## Licence

Ce projet est un exemple éducatif.

