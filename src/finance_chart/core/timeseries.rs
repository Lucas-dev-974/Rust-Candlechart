use super::candle::Candle;
use super::cache::{PriceRangeCache, TimeRangeCache};
use std::ops::Range;

/// Série temporelle de bougies OHLC
/// 
/// Gère une collection ordonnée de bougies avec des opérations
/// d'agrégation et de recherche efficaces.
#[derive(Debug)]
pub struct TimeSeries {
    candles: Vec<Candle>,
    /// Cache pour les plages de prix
    price_cache: PriceRangeCache,
    /// Cache pour les plages temporelles
    time_cache: TimeRangeCache,
}

impl TimeSeries {
    /// Crée une nouvelle série temporelle vide
    pub fn new() -> Self {
        Self {
            candles: Vec::new(),
            price_cache: PriceRangeCache::new(),
            time_cache: TimeRangeCache::new(),
        }
    }

    /// Ajoute une bougie à la série
    /// Invalide automatiquement les caches
    pub fn push(&mut self, candle: Candle) {
        self.candles.push(candle);
        // Invalider les caches car les données ont changé
        self.price_cache.invalidate();
        self.time_cache.invalidate();
    }

    /// Retourne le nombre de bougies dans la série
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Retourne le timestamp minimum
    pub fn min_timestamp(&self) -> Option<i64> {
        // Vérifier le cache d'abord
        if let Some((min, _)) = self.time_cache.get_global() {
            return Some(min);
        }
        
        // Calculer et mettre en cache
        let result = self.candles.first().map(|c| c.timestamp);
        if let Some(min) = result {
            if let Some(max) = self.candles.last().map(|c| c.timestamp) {
                self.time_cache.set_global((min, max));
            }
        }
        result
    }

    /// Retourne le timestamp maximum
    pub fn max_timestamp(&self) -> Option<i64> {
        // Vérifier le cache d'abord
        if let Some((_, max)) = self.time_cache.get_global() {
            return Some(max);
        }
        
        // Calculer et mettre en cache
        let result = self.candles.last().map(|c| c.timestamp);
        if let Some(max) = result {
            if let Some(min) = self.candles.first().map(|c| c.timestamp) {
                self.time_cache.set_global((min, max));
            }
        }
        result
    }

    /// Retourne la dernière bougie (prix courant)
    pub fn last_candle(&self) -> Option<&Candle> {
        self.candles.last()
    }

    /// Retourne la plage de prix (min, max) en une seule itération
    /// Utilise le cache si disponible
    pub fn price_range(&self) -> Option<(f64, f64)> {
        // Vérifier le cache d'abord
        if let Some(cached) = self.price_cache.get_global() {
            return Some(cached);
        }
        
        // Calculer et mettre en cache
        let result: Option<(f64, f64)> = self.candles.iter().fold(None, |acc, candle| {
            Some(match acc {
                None => (candle.low, candle.high),
                Some((min, max)) => (min.min(candle.low), max.max(candle.high)),
            })
        });
        
        if let Some(range) = result {
            self.price_cache.set_global(range);
        }
        
        result
    }

    /// Retourne la plage de prix pour une plage temporelle spécifique
    /// Utilise le cache si disponible
    pub fn price_range_for_time_range(&self, time_range: Range<i64>) -> Option<(f64, f64)> {
        // Vérifier le cache d'abord
        if let Some(cached) = self.price_cache.get_range(time_range.clone()) {
            return Some(cached);
        }
        
        // Calculer pour les bougies visibles
        let visible = self.visible_candles(time_range.clone());
        let result: Option<(f64, f64)> = visible.iter().fold(None, |acc, candle| {
            Some(match acc {
                None => (candle.low, candle.high),
                Some((min, max)) => (min.min(candle.low), max.max(candle.high)),
            })
        });
        
        // Mettre en cache le résultat
        if let Some(range) = result {
            self.price_cache.set_range(time_range, range);
        }
        
        result
    }

    /// Retourne les bougies visibles dans une plage de timestamps
    /// 
    /// Utilise une recherche binaire pour efficacité avec grandes séries
    pub fn visible_candles(&self, time_range: Range<i64>) -> &[Candle] {
        if self.candles.is_empty() {
            return &[];
        }

        // Recherche binaire pour trouver le début
        let start_idx = self
            .candles
            .binary_search_by_key(&time_range.start, |c| c.timestamp)
            .unwrap_or_else(|idx| idx);

        // Recherche binaire pour trouver la fin
        let end_idx = self
            .candles
            .binary_search_by_key(&time_range.end, |c| c.timestamp)
            .map(|idx| idx + 1)
            .unwrap_or_else(|idx| idx);

        &self.candles[start_idx.min(self.candles.len())..end_idx.min(self.candles.len())]
    }
}

impl Default for TimeSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TimeSeries {
    fn clone(&self) -> Self {
        // Cloner les bougies mais créer de nouveaux caches vides
        // (les Cell ne sont pas Clone, et de toute façon le cache sera recalculé)
        Self {
            candles: self.candles.clone(),
            price_cache: PriceRangeCache::new(),
            time_cache: TimeRangeCache::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeseries_basic() {
        let mut ts = TimeSeries::new();

        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0));
        ts.push(Candle::new(2000, 104.0, 106.0, 103.0, 105.0));

        assert_eq!(ts.min_timestamp(), Some(1000));
        assert_eq!(ts.max_timestamp(), Some(2000));
        assert_eq!(ts.price_range(), Some((99.0, 106.0)));
    }

    #[test]
    fn test_visible_candles() {
        let mut ts = TimeSeries::new();
        for i in 0..10 {
            ts.push(Candle::new(i * 1000, 100.0, 105.0, 99.0, 104.0));
        }

        let visible = ts.visible_candles(2500..5500);
        assert_eq!(visible.len(), 3); // indices 2, 3, 4
    }

    #[test]
    fn test_price_range_cache() {
        let mut ts = TimeSeries::new();
        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0));
        ts.push(Candle::new(2000, 104.0, 110.0, 103.0, 108.0));
        ts.push(Candle::new(3000, 108.0, 115.0, 107.0, 112.0));

        // Premier appel : doit calculer
        let range1 = ts.price_range();
        assert_eq!(range1, Some((99.0, 115.0)));

        // Deuxième appel : doit utiliser le cache
        let range2 = ts.price_range();
        assert_eq!(range2, Some((99.0, 115.0)));
        assert_eq!(range1, range2);
    }

    #[test]
    fn test_price_range_cache_invalidation() {
        let mut ts = TimeSeries::new();
        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0));
        
        let range1 = ts.price_range();
        assert_eq!(range1, Some((99.0, 105.0)));

        // Ajouter une nouvelle bougie : le cache doit être invalidé
        ts.push(Candle::new(2000, 110.0, 120.0, 109.0, 115.0));
        
        let range2 = ts.price_range();
        assert_eq!(range2, Some((99.0, 120.0))); // Doit inclure la nouvelle bougie
    }

    #[test]
    fn test_price_range_for_time_range_cache() {
        let mut ts = TimeSeries::new();
        for i in 0..10 {
            ts.push(Candle::new(i * 1000, 100.0 + i as f64, 105.0 + i as f64, 99.0 + i as f64, 104.0 + i as f64));
        }

        let time_range = 2000..6000;
        
        // Premier appel : doit calculer
        let range1 = ts.price_range_for_time_range(time_range.clone());
        assert!(range1.is_some());

        // Deuxième appel : doit utiliser le cache
        let range2 = ts.price_range_for_time_range(time_range);
        assert_eq!(range1, range2);
    }
}

