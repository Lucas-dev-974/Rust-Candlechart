# Analyse et Plan de Refactorisation

## Problèmes Identifiés

### 1. **Fichier main.rs trop volumineux (1809 lignes)**
   - **Problème** : Un seul fichier contient toute la logique de l'application
   - **Impact** : Difficile à maintenir, naviguer et tester
   - **Solution** : Découper en modules logiques

### 2. **Duplication de code**
   - **Problème** : `calculate_candles_back_timestamp` et `interval_to_seconds` font essentiellement la même chose
   - **Impact** : Maintenance difficile, risque d'incohérence
   - **Solution** : Unifier en utilisant `interval_to_seconds` comme fonction de base

### 3. **Méthode `update()` trop longue (~300 lignes)**
   - **Problème** : Un seul match géant avec beaucoup de branches
   - **Impact** : Difficile à comprendre et maintenir
   - **Solution** : Extraire les handlers dans des modules séparés

### 4. **Méthodes très longues**
   - `complete_missing_data()` : ~90 lignes
   - `complete_gaps()` : ~70 lignes
   - `apply_complete_gaps_results()` : ~110 lignes
   - **Solution** : Extraire dans un module `realtime` ou `data_sync`

### 5. **Logique de sauvegarde complexe**
   - La sauvegarde dans `apply_complete_gaps_results` est très verbeuse
   - **Solution** : Créer une fonction helper dédiée

## Plan de Refactorisation

### Phase 1 : Simplifications immédiates ✅
- [x] Créer `app/utils.rs` pour les fonctions utilitaires
- [x] Créer `app/window_manager.rs` pour la gestion des fenêtres
- [x] Créer `app/constants.rs` pour les constantes
- [ ] Simplifier `calculate_candles_back_timestamp` pour utiliser `interval_to_seconds`

### Phase 2 : Extraction des messages et état
- [ ] Créer `app/messages.rs` pour l'enum Message
- [ ] Créer `app/app_state.rs` pour la structure ChartApp

### Phase 3 : Extraction de la logique métier
- [ ] Créer `app/data_loading.rs` pour le chargement asynchrone
- [ ] Créer `app/realtime.rs` pour toute la logique temps réel
- [ ] Créer `app/handlers.rs` pour les handlers de messages

### Phase 4 : Extraction des vues
- [ ] Créer `app/views.rs` pour toutes les méthodes view

## Recommandations de Simplification

1. **Unifier les fonctions d'intervalle** : Utiliser `interval_to_seconds` partout
2. **Extraire les helpers de sauvegarde** : Créer `save_series_async()` helper
3. **Simplifier les handlers** : Grouper les handlers par domaine (windows, settings, providers, realtime)

