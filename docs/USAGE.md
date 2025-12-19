# Guide d'Utilisation

## Table des matières

1. [Installation](#installation)
2. [Premier lancement](#premier-lancement)
3. [Navigation](#navigation)
4. [Dessin](#dessin)
5. [Édition](#édition)
6. [Sélection de série](#sélection-de-série)
7. [Personnalisation](#personnalisation)
8. [Raccourcis clavier](#raccourcis-clavier)
9. [Format des données](#format-des-données)

---

## Installation

### Prérequis

- Rust (version 1.70+)
- Cargo (généralement inclus avec Rust)

### Compilation

```bash
# Cloner ou télécharger le projet
cd CandleChart

# Compiler
cargo build

# Compiler en mode release (optimisé)
cargo build --release
```

### Exécution

```bash
# Mode debug
cargo run

# Mode release
cargo run --release
```

---

## Premier lancement

### Préparation des données

1. Créer un dossier `data/` à la racine du projet
2. Placer vos fichiers JSON dans ce dossier
3. Format attendu : Voir [Format des données](#format-des-données)

### Au démarrage

L'application :
- Charge automatiquement tous les fichiers JSON du dossier `data/`
- Active la première série trouvée
- Charge les dessins sauvegardés depuis `drawings.json` (si présent)
- Charge le style depuis `chart_style.json` (si présent)

---

## Navigation

### Pan (Déplacement)

**Clic gauche + Drag** : Déplace le graphique horizontalement et verticalement

- Maintenez le bouton gauche de la souris enfoncé
- Déplacez la souris
- Relâchez pour terminer

### Zoom

#### Zoom horizontal (axe X / temps)

**Molette** : Zoom avant/arrière sur l'axe temporel

- Molette vers le haut : Zoom in (moins de temps visible)
- Molette vers le bas : Zoom out (plus de temps visible)

#### Zoom vertical (axe Y / prix)

**ALT + Molette** : Zoom avant/arrière sur l'axe des prix

- ALT + Molette vers le haut : Zoom in (plage de prix plus petite)
- ALT + Molette vers le bas : Zoom out (plage de prix plus grande)

#### Zoom sur les deux axes

**CTRL + Molette** : Zoom simultané sur les deux axes

### Zoom depuis les axes

- **Axe Y (prix)** : Clic gauche + drag vertical pour zoomer
- **Axe X (temps)** : Clic gauche + drag horizontal pour zoomer

---

## Dessin

### Outils disponibles

1. **Rectangle** : Dessine un rectangle sur le graphique
2. **Ligne horizontale** : Dessine une ligne horizontale à un prix spécifique

### Utilisation

1. **Sélectionner un outil** :
   - Cliquer sur l'icône de l'outil dans le panel de gauche
   - L'outil sélectionné est surligné

2. **Dessiner** :
   - **Rectangle** : Clic gauche + drag pour définir la zone
   - **Ligne horizontale** : Clic gauche à la hauteur de prix désirée

3. **Annuler un dessin en cours** :
   - Appuyer sur `ESC` ou cliquer ailleurs

### Couleurs

Les rectangles et lignes utilisent des couleurs par défaut :
- Rectangles : Bleu semi-transparent
- Lignes horizontales : Rouge semi-transparent

---

## Édition

### Sélectionner un élément

**Clic gauche** sur un rectangle ou une ligne horizontale pour le sélectionner.

### Déplacer un rectangle

1. Sélectionner le rectangle
2. **Clic gauche + drag** au centre du rectangle
3. Relâcher pour terminer

### Redimensionner un rectangle

1. Sélectionner le rectangle
2. **Clic gauche + drag** sur :
   - Un coin : Redimensionne depuis ce coin
   - Un bord : Redimensionne depuis ce bord
3. Relâcher pour terminer

### Déplacer une ligne horizontale

1. Sélectionner la ligne
2. **Clic gauche + drag** verticalement
3. Relâcher pour terminer

### Désélectionner

- **Clic gauche** ailleurs sur le graphique
- Appuyer sur `ESC`

---

## Sélection de série

### Utiliser le select box

1. Cliquer sur le select box en haut à droite du graphique
2. Choisir une série dans la liste
3. Le graphique se met à jour automatiquement avec :
   - Le zoom réinitialisé
   - Les données de la nouvelle série

### Affichage

- Le titre du graphique affiche le symbole de la série active
- Le titre de la fenêtre affiche également le symbole

---

## Personnalisation

### Modifier les couleurs

1. Cliquer sur l'icône ⚙ en bas à droite
2. La fenêtre de settings s'ouvre
3. Cliquer sur une couleur pour ouvrir le sélecteur
4. Choisir une couleur parmi les presets ou utiliser la couleur actuelle
5. Cliquer sur **"Appliquer"** pour sauvegarder

### Couleurs personnalisables

- Fond du graphique
- Bougie haussière
- Bougie baissière
- Mèches
- Grille
- Prix courant
- Crosshair
- Texte

### Sauvegarde automatique

Les styles sont automatiquement sauvegardés dans `chart_style.json` lors du clic sur "Appliquer".

---

## Raccourcis clavier

| Touche | Action |
|--------|--------|
| `ESC` | Annuler le dessin en cours / Désélectionner |
| `CTRL + Z` | Annuler la dernière action (Undo) |
| `CTRL + Y` | Rétablir la dernière action annulée (Redo) |
| `DELETE` / `SUPPR` | Supprimer l'élément sélectionné |
| `SHIFT` (maintenu) | Afficher le tooltip OHLC sous la souris |
| `CTRL + S` | Sauvegarder les dessins (à implémenter) |
| `CTRL + O` | Charger les dessins (à implémenter) |

### Modificateurs pour le zoom

| Modificateur | Action |
|--------------|--------|
| Aucun | Zoom horizontal (molette) |
| `ALT` | Zoom vertical (molette) |
| `CTRL` | Zoom sur les deux axes (molette) |

---

## Format des données

### Structure JSON

Les fichiers JSON doivent suivre ce format :

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

### Champs requis

- `symbol` : Symbole de la paire (ex: "BTCUSDT")
- `interval` : Intervalle temporel (ex: "1h", "15m", "1d")
- `klines` : Tableau de bougies

### Format des bougies

- `open_time` : Timestamp d'ouverture en **millisecondes** (sera converti en secondes)
- `open` : Prix d'ouverture
- `high` : Prix le plus haut
- `low` : Prix le plus bas
- `close` : Prix de clôture
- `volume` : Volume (non utilisé mais requis)

### Exemple de fichier

Voir `data/BTCUSDT_1h.json` pour un exemple complet.

---

## Conseils d'utilisation

### Performance

- Pour de grandes séries (> 100k bougies), le cache optimise automatiquement les calculs
- Le zoom permet de visualiser différentes périodes efficacement

### Organisation

- Utilisez des noms de fichiers descriptifs : `SYMBOL_INTERVAL.json`
- Exemple : `BTCUSDT_1h.json`, `ETHUSDT_15m.json`

### Sauvegarde

- Les dessins sont sauvegardés dans `drawings.json`
- Les styles sont sauvegardés dans `chart_style.json`
- Ces fichiers sont créés automatiquement lors de la première sauvegarde

### Multi-séries

- Une seule série est affichée à la fois
- Changez de série via le select box pour comparer différentes données
- Le zoom se réinitialise automatiquement lors du changement de série

---

## Dépannage

### Le graphique ne s'affiche pas

1. Vérifier que le dossier `data/` contient des fichiers JSON valides
2. Vérifier le format JSON (voir [Format des données](#format-des-données))
3. Consulter la console pour les messages d'erreur

### Les dessins ne se sauvegardent pas

1. Vérifier les permissions d'écriture dans le répertoire du projet
2. Consulter la console pour les messages d'erreur

### Le zoom ne fonctionne pas

1. Vérifier que la souris est sur le graphique (pas sur les axes)
2. Essayer avec différents modificateurs (ALT, CTRL)

### Les couleurs ne changent pas

1. Vérifier que vous avez cliqué sur "Appliquer" dans les settings
2. Vérifier que `chart_style.json` est bien créé et modifiable

