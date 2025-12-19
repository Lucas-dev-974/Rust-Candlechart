//! Système de cache pour les calculs de plages
//!
//! Évite de recalculer les plages de prix et de temps
//! qui sont coûteuses avec de grandes séries de données.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Range;

/// Cache pour les plages de prix calculées
/// Utilise Cell/RefCell pour permettre la mutation interne avec des références immuables
#[derive(Debug, Default)]
pub struct PriceRangeCache {
    /// Cache global de la plage de prix (toute la série)
    global_price_range: Cell<Option<(f64, f64)>>,
    /// Cache des plages de prix pour des plages temporelles spécifiques
    /// Clé: (min_time, max_time), Valeur: (min_price, max_price)
    range_cache: RefCell<HashMap<(i64, i64), (f64, f64)>>,
}

impl PriceRangeCache {
    /// Crée un nouveau cache vide
    pub fn new() -> Self {
        Self {
            global_price_range: Cell::new(None),
            range_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Invalide tout le cache (appelé quand les données changent)
    pub fn invalidate(&self) {
        self.global_price_range.set(None);
        self.range_cache.borrow_mut().clear();
    }

    /// Récupère la plage de prix globale depuis le cache
    pub fn get_global(&self) -> Option<(f64, f64)> {
        self.global_price_range.get()
    }

    /// Met en cache la plage de prix globale
    pub fn set_global(&self, range: (f64, f64)) {
        self.global_price_range.set(Some(range));
    }

    /// Récupère une plage de prix pour une plage temporelle spécifique
    pub fn get_range(&self, time_range: Range<i64>) -> Option<(f64, f64)> {
        self.range_cache.borrow().get(&(time_range.start, time_range.end)).copied()
    }

    /// Met en cache une plage de prix pour une plage temporelle spécifique
    pub fn set_range(&self, time_range: Range<i64>, price_range: (f64, f64)) {
        let mut cache = self.range_cache.borrow_mut();
        cache.insert((time_range.start, time_range.end), price_range);
        
        // Limiter la taille du cache pour éviter une consommation mémoire excessive
        const MAX_CACHE_SIZE: usize = 100;
        if cache.len() > MAX_CACHE_SIZE {
            // Supprimer les entrées les plus anciennes (simple stratégie FIFO)
            // En pratique, on pourrait utiliser un LRU, mais pour l'instant on vide simplement
            cache.clear();
        }
    }
}

/// Cache pour les plages temporelles calculées
/// Utilise Cell pour permettre la mutation interne avec des références immuables
#[derive(Debug, Default)]
pub struct TimeRangeCache {
    /// Cache global de la plage temporelle (toute la série)
    global_time_range: Cell<Option<(i64, i64)>>,
}

impl TimeRangeCache {
    /// Crée un nouveau cache vide
    pub fn new() -> Self {
        Self {
            global_time_range: Cell::new(None),
        }
    }

    /// Invalide tout le cache (appelé quand les données changent)
    pub fn invalidate(&self) {
        self.global_time_range.set(None);
    }

    /// Récupère la plage temporelle globale depuis le cache
    pub fn get_global(&self) -> Option<(i64, i64)> {
        self.global_time_range.get()
    }

    /// Met en cache la plage temporelle globale
    pub fn set_global(&self, range: (i64, i64)) {
        self.global_time_range.set(Some(range));
    }
}

