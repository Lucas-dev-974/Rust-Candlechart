# CrateTeste - Test du crate CandleChart

Ce projet teste l'utilisation de CandleChart comme librairie externe avec **deux approches** : API Simplifiée et API Avancée.

## Description

Ce projet démontre comment utiliser le crate `candlechart` dans un projet externe avec deux niveaux de complexité.

## Structure

```
crateTeste/
├── Cargo.toml           # Dépendance vers CandleChart (chemin local)
├── src/
│   ├── main.rs          # Application de test complète (API Avancée)
│   ├── main_simple.rs  # Application ultra-simple (API Simplifiée - 3 lignes !)
│   └── main_advanced.rs # Application avancée (API Avancée complète)
└── README.md            # Ce fichier
```

## Utilisation

### API Simplifiée (3 lignes !)

```bash
# Exécuter l'exemple simple
cargo run --manifest-path crateTeste/Cargo.toml --bin simple
```

**Code** (`main_simple.rs`) :
```rust
use candlechart::simple_app;

fn main() -> iced::Result {
    simple_app("../data", 1200.0, 800.0)
}
```

### API Avancée (personnalisation complète)

```bash
# Exécuter l'exemple avancé
cargo run --manifest-path crateTeste/Cargo.toml --bin advanced

# Ou l'exemple complet
cargo run --manifest-path crateTeste/Cargo.toml
```

## Comparaison des approches

### API Simplifiée ✅

**Avantages** :
- ⚡ **3 lignes de code** seulement
- 🚀 Démarrage rapide
- 📦 Tout est géré automatiquement
- 🎯 Parfait pour les cas d'usage basiques

**Code** :
```rust
use candlechart::simple_app;

fn main() -> iced::Result {
    simple_app("../data", 1200.0, 800.0)
}
```

### API Avancée ✅

**Avantages** :
- 🎨 Personnalisation complète
- 🔧 Contrôle total sur les messages
- 🪟 Support multi-fenêtres
- 🎛️ Intégration dans des applications complexes

**Code** : Voir `main_advanced.rs` ou `main.rs` (~250 lignes)

## Fonctionnalités testées

### API Simplifiée
- ✅ Chargement automatique des données
- ✅ Interface complète fonctionnelle
- ✅ Navigation (pan, zoom)
- ✅ Changement de série

### API Avancée
- ✅ Toutes les fonctionnalités de l'API Simplifiée
- ✅ Gestion manuelle des messages
- ✅ Personnalisation de l'UI
- ✅ Contrôle fin du comportement

## Dépendances

- `candlechart` : Le crate CandleChart (chemin local `../`)
- `iced` : Framework GUI (version 0.14)

## Notes

- Les données sont chargées depuis `../data/` (dossier du projet parent)
- L'API Simplifiée est idéale pour démarrer rapidement
- L'API Avancée permet une personnalisation complète
- Vous pouvez migrer de l'API Simplifiée vers l'API Avancée progressivement

