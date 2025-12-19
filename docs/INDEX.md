# Index de la Documentation

## 📚 Navigation rapide

### Documentation principale

1. **[README.md](./README.md)** - Vue d'ensemble et point d'entrée
   - Introduction au projet
   - Technologies utilisées
   - Structure du projet
   - Liens vers toutes les sections

2. **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Architecture détaillée
   - Pattern Elm Architecture
   - Architecture modulaire
   - Flux de données
   - Diagrammes d'architecture
   - Séparation des responsabilités
   - Système de cache
   - Gestion des messages

3. **[MODULES.md](./MODULES.md)** - Documentation des modules
   - Core (candle, timeseries, series_data, cache)
   - Scale (price, time)
   - Viewport
   - Render (candlestick, grid, crosshair, tooltip, etc.)
   - Interaction (events, rectangle_editing)
   - State (ChartState)
   - Widget
   - Data Loader
   - Settings
   - Messages

4. **[API.md](./API.md)** - Référence API complète
   - API publique
   - Core API (Candle, TimeSeries, SeriesManager)
   - Viewport API (Viewport, PriceScale, TimeScale)
   - Render API (fonctions de rendu)
   - State API (ChartState)
   - Constantes

5. **[DATA_STRUCTURES.md](./DATA_STRUCTURES.md)** - Structures de données
   - Structures Core
   - Structures de Scale
   - Structures de Viewport
   - Structures de Render
   - Structures d'Interaction
   - Structures d'État
   - Structures de Tools
   - Structures de Settings
   - Diagramme de relations
   - Formats de sérialisation

6. **[USAGE.md](./USAGE.md)** - Guide d'utilisation
   - Installation
   - Premier lancement
   - Navigation (pan, zoom)
   - Dessin (rectangles, lignes)
   - Édition (déplacement, redimensionnement)
   - Sélection de série
   - Personnalisation
   - Raccourcis clavier
   - Format des données
   - Dépannage

7. **[FLOW_DIAGRAMS.md](./FLOW_DIAGRAMS.md)** - Diagrammes de flux
   - Flux de chargement
   - Flux d'interaction (pan, zoom)
   - Flux de rendu
   - Flux de dessin
   - Flux d'édition
   - Flux de changement de série
   - Diagrammes de séquence
   - Optimisations

---

## 🎯 Parcours recommandés

### Pour les nouveaux développeurs

1. Commencer par [README.md](./README.md) pour comprendre le projet
2. Lire [ARCHITECTURE.md](./ARCHITECTURE.md) pour comprendre l'architecture
3. Consulter [MODULES.md](./MODULES.md) pour connaître les modules
4. Utiliser [API.md](./API.md) comme référence lors du développement

### Pour les utilisateurs

1. Commencer par [README.md](./README.md)
2. Lire [USAGE.md](./USAGE.md) pour apprendre à utiliser l'application
3. Consulter [DATA_STRUCTURES.md](./DATA_STRUCTURES.md) pour comprendre le format des données

### Pour comprendre le code

1. [ARCHITECTURE.md](./ARCHITECTURE.md) - Vue d'ensemble
2. [MODULES.md](./MODULES.md) - Détails des modules
3. [FLOW_DIAGRAMS.md](./FLOW_DIAGRAMS.md) - Flux d'exécution
4. [API.md](./API.md) - Référence des fonctions

---

## 📊 Schémas et diagrammes

### Diagrammes d'architecture

- **ARCHITECTURE.md** : Contient des diagrammes ASCII art pour :
  - Pattern Elm Architecture
  - Architecture modulaire
  - Flux de données
  - Gestion des messages

### Diagrammes de flux

- **FLOW_DIAGRAMS.md** : Contient des diagrammes détaillés pour :
  - Chargement initial
  - Interactions utilisateur (pan, zoom)
  - Cycle de rendu
  - Dessin et édition
  - Changement de série

### Diagrammes de relations

- **DATA_STRUCTURES.md** : Contient un diagramme montrant les relations entre les structures de données

---

## 🔍 Recherche rapide

### Par sujet

| Sujet | Fichier |
|-------|---------|
| Installation | [USAGE.md](./USAGE.md) |
| Navigation | [USAGE.md](./USAGE.md) |
| Dessin | [USAGE.md](./USAGE.md) |
| Architecture | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| Modules | [MODULES.md](./MODULES.md) |
| API | [API.md](./API.md) |
| Structures | [DATA_STRUCTURES.md](./DATA_STRUCTURES.md) |
| Flux | [FLOW_DIAGRAMS.md](./FLOW_DIAGRAMS.md) |

### Par type de contenu

| Type | Fichiers |
|------|----------|
| Schémas/Diagrammes | ARCHITECTURE.md, FLOW_DIAGRAMS.md, DATA_STRUCTURES.md |
| Référence technique | API.md, MODULES.md, DATA_STRUCTURES.md |
| Guide utilisateur | USAGE.md, README.md |
| Concepts | ARCHITECTURE.md, MODULES.md |

---

## 📝 Notes

- Tous les fichiers sont en format Markdown (.md)
- Les diagrammes utilisent la syntaxe ASCII art
- Les exemples de code sont en Rust
- Les formats JSON sont documentés dans DATA_STRUCTURES.md

---

## 🔗 Liens externes

- [Documentation Rust](https://doc.rust-lang.org/)
- [Iced Framework](https://docs.rs/iced/)
- [Serde](https://serde.rs/)

