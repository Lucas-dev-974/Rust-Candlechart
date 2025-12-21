use super::candle::Candle;
use super::cache::{PriceRangeCache, TimeRangeCache};
use std::ops::Range;

/// Erreur de validation d'une bougie
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Timestamp invalide (négatif ou trop dans le futur)
    InvalidTimestamp,
    /// Prix invalide (négatif, NaN ou infini)
    InvalidPrice(String),
    /// Incohérence OHLC (high < low, ou high/low ne contient pas open/close)
    InvalidOHLC,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidTimestamp => write!(f, "Timestamp invalide"),
            ValidationError::InvalidPrice(field) => write!(f, "Prix invalide pour {}: doit être positif et fini", field),
            ValidationError::InvalidOHLC => write!(f, "Incohérence OHLC: high doit être >= max(open,close) et low <= min(open,close)"),
        }
    }
}

impl std::error::Error for ValidationError {}

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

    /// Valide une bougie avant insertion
    ///
    /// Vérifie :
    /// - Timestamp valide (positif, pas trop dans le futur)
    /// - Prix valides (positifs, finis, pas NaN)
    /// - Cohérence OHLC (high >= max(open,close), low <= min(open,close))
    fn validate_candle(candle: &Candle) -> Result<(), ValidationError> {
        // Vérifier le timestamp (doit être positif et pas trop dans le futur)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        if candle.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        // Permettre jusqu'à 1 heure dans le futur (pour les bougies en cours)
        if candle.timestamp > now + 3600 {
            return Err(ValidationError::InvalidTimestamp);
        }

        // Vérifier que les prix sont valides (positifs, finis, pas NaN)
        let check_price = |value: f64, field: &str| -> Result<(), ValidationError> {
            if value.is_nan() || value.is_infinite() || value < 0.0 {
                return Err(ValidationError::InvalidPrice(field.to_string()));
            }
            Ok(())
        };

        check_price(candle.open, "open")?;
        check_price(candle.high, "high")?;
        check_price(candle.low, "low")?;
        check_price(candle.close, "close")?;

        // Vérifier la cohérence OHLC
        let max_price = candle.open.max(candle.close);
        let min_price = candle.open.min(candle.close);
        
        if candle.high < max_price || candle.low > min_price {
            return Err(ValidationError::InvalidOHLC);
        }

        Ok(())
    }

    /// Ajoute une bougie à la série
    /// Invalide automatiquement les caches
    ///
    /// # Erreurs
    /// Retourne une erreur si la bougie est invalide
    pub fn push(&mut self, candle: Candle) -> Result<(), ValidationError> {
        Self::validate_candle(&candle)?;
        self.candles.push(candle);
        // Invalider les caches car les données ont changé
        self.price_cache.invalidate();
        self.time_cache.invalidate();
        Ok(())
    }

    /// Met à jour la dernière bougie si elle a le même timestamp, sinon ajoute une nouvelle bougie
    ///
    /// Utile pour les mises à jour en temps réel où la bougie courante peut être mise à jour
    /// avant sa clôture définitive.
    ///
    /// Gère correctement les cas où les bougies arrivent dans le désordre (par exemple,
    /// lorsque plusieurs requêtes HTTP sont en vol simultanément et se terminent dans un ordre différent).
    /// Dans ce cas, utilise `merge_candles` pour maintenir l'ordre chronologique.
    ///
    /// # Retourne
    /// - `Ok(true)` si la bougie a été mise à jour (même timestamp)
    /// - `Ok(false)` si une nouvelle bougie a été ajoutée (nouveau timestamp)
    /// - `Err(ValidationError)` si la bougie est invalide
    pub fn update_or_append_candle(&mut self, candle: Candle) -> Result<bool, ValidationError> {
        Self::validate_candle(&candle)?;
        
        // Vérifier si la dernière bougie a le même timestamp
        if let Some(last) = self.candles.last() {
            if last.timestamp == candle.timestamp {
                // Mettre à jour la bougie existante
                if let Some(last_mut) = self.candles.last_mut() {
                    *last_mut = candle;
                    self.price_cache.invalidate();
                    self.time_cache.invalidate();
                    return Ok(true);
                }
            } else if last.timestamp > candle.timestamp {
                // La bougie arrivée est plus ancienne que la dernière bougie
                // Cela peut arriver si plusieurs requêtes HTTP sont en vol simultanément
                // et se terminent dans le désordre. Utiliser merge_candles pour maintenir
                // l'ordre chronologique.
                self.merge_candles(vec![candle]);
                return Ok(false);
            }
        }
        
        // Ajouter une nouvelle bougie (timestamp > dernière bougie, ou série vide)
        self.push(candle)?;
        Ok(false)
    }

    /// Fusionne des bougies dans la série en évitant les doublons
    ///
    /// Les bougies avec le même timestamp remplacent les existantes.
    /// Les nouvelles bougies sont insérées dans l'ordre chronologique.
    /// Les bougies invalides sont ignorées (avec un log d'avertissement).
    ///
    /// # Retourne
    /// Le nombre de bougies ajoutées (pas mises à jour)
    pub fn merge_candles(&mut self, new_candles: Vec<Candle>) -> usize {
        if new_candles.is_empty() {
            return 0;
        }

        let mut added_count = 0;
        
        for new_candle in new_candles {
            // Valider la bougie avant insertion
            if let Err(e) = Self::validate_candle(&new_candle) {
                eprintln!("⚠️ Bougie invalide ignorée: {} (timestamp: {})", e, new_candle.timestamp);
                continue;
            }
            
            // Chercher si une bougie avec le même timestamp existe déjà
            let existing_idx = self.candles
                .binary_search_by_key(&new_candle.timestamp, |c| c.timestamp)
                .ok();
            
            match existing_idx {
                Some(idx) => {
                    // Remplacer la bougie existante
                    self.candles[idx] = new_candle;
                }
                None => {
                    // Insérer à la bonne position pour maintenir l'ordre
                    let insert_idx = self.candles
                        .binary_search_by_key(&new_candle.timestamp, |c| c.timestamp)
                        .unwrap_or_else(|idx| idx);
                    self.candles.insert(insert_idx, new_candle);
                    added_count += 1;
                }
            }
        }
        
        // Invalider les caches
        self.price_cache.invalidate();
        self.time_cache.invalidate();
        
        added_count
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

    /// Retourne toutes les bougies de la série
    pub fn all_candles(&self) -> &[Candle] {
        &self.candles
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

    /// Détecte les gaps dans les données selon l'intervalle attendu
    /// 
    /// # Arguments
    /// * `expected_interval_seconds` - L'intervalle attendu entre deux bougies consécutives (en secondes)
    /// 
    /// # Retourne
    /// Un vecteur de tuples (start_timestamp, end_timestamp) représentant les gaps détectés.
    /// Un gap est détecté si l'intervalle entre deux bougies consécutives est supérieur à 1.5 fois l'intervalle attendu.
    pub fn detect_gaps(&self, expected_interval_seconds: i64) -> Vec<(i64, i64)> {
        if self.candles.len() < 2 {
            return Vec::new();
        }

        let mut gaps = Vec::new();
        let threshold = (expected_interval_seconds as f64 * 1.5) as i64;

        for i in 0..(self.candles.len() - 1) {
            let current_ts = self.candles[i].timestamp;
            let next_ts = self.candles[i + 1].timestamp;
            let actual_interval = next_ts - current_ts;

            // Si l'intervalle réel est significativement plus grand que l'intervalle attendu, c'est un gap
            if actual_interval > threshold {
                // Le gap commence juste après la bougie actuelle et se termine juste avant la bougie suivante
                // On ajoute l'intervalle attendu pour obtenir le timestamp de début du gap
                let gap_start = current_ts + expected_interval_seconds;
                let gap_end = next_ts - expected_interval_seconds;
                
                // S'assurer que gap_start < gap_end
                if gap_start < gap_end {
                    gaps.push((gap_start, gap_end));
                }
            }
        }

        gaps
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

        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0)).unwrap();
        ts.push(Candle::new(2000, 104.0, 106.0, 103.0, 105.0)).unwrap();

        assert_eq!(ts.min_timestamp(), Some(1000));
        assert_eq!(ts.max_timestamp(), Some(2000));
        assert_eq!(ts.price_range(), Some((99.0, 106.0)));
    }

    #[test]
    fn test_visible_candles() {
        let mut ts = TimeSeries::new();
        for i in 0..10 {
            ts.push(Candle::new(i * 1000, 100.0, 105.0, 99.0, 104.0)).unwrap();
        }

        let visible = ts.visible_candles(2500..5500);
        assert_eq!(visible.len(), 3); // indices 2, 3, 4
    }

    #[test]
    fn test_price_range_cache() {
        let mut ts = TimeSeries::new();
        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0)).unwrap();
        ts.push(Candle::new(2000, 104.0, 110.0, 103.0, 108.0)).unwrap();
        ts.push(Candle::new(3000, 108.0, 115.0, 107.0, 112.0)).unwrap();

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
        ts.push(Candle::new(1000, 100.0, 105.0, 99.0, 104.0)).unwrap();
        
        let range1 = ts.price_range();
        assert_eq!(range1, Some((99.0, 105.0)));

        // Ajouter une nouvelle bougie : le cache doit être invalidé
        ts.push(Candle::new(2000, 110.0, 120.0, 109.0, 115.0)).unwrap();
        
        let range2 = ts.price_range();
        assert_eq!(range2, Some((99.0, 120.0))); // Doit inclure la nouvelle bougie
    }

    #[test]
    fn test_price_range_for_time_range_cache() {
        let mut ts = TimeSeries::new();
        for i in 0..10 {
            ts.push(Candle::new(i * 1000, 100.0 + i as f64, 105.0 + i as f64, 99.0 + i as f64, 104.0 + i as f64)).unwrap();
        }

        let time_range = 2000..6000;
        
        // Premier appel : doit calculer
        let range1 = ts.price_range_for_time_range(time_range.clone());
        assert!(range1.is_some());

        // Deuxième appel : doit utiliser le cache
        let range2 = ts.price_range_for_time_range(time_range);
        assert_eq!(range1, range2);
    }

    #[test]
    fn test_update_or_append_out_of_order() {
        // Test pour vérifier que les bougies arrivant dans le désordre sont gérées correctement
        // Simule le scénario où plusieurs requêtes HTTP sont en vol simultanément
        let mut ts = TimeSeries::new();
        
        // Ajouter une bougie à t=100
        ts.update_or_append_candle(Candle::new(100, 100.0, 105.0, 99.0, 104.0)).unwrap();
        assert_eq!(ts.len(), 1);
        assert_eq!(ts.min_timestamp(), Some(100));
        assert_eq!(ts.max_timestamp(), Some(100));
        
        // Ajouter une bougie à t=160 (plus récente)
        ts.update_or_append_candle(Candle::new(160, 104.0, 106.0, 103.0, 105.0)).unwrap();
        assert_eq!(ts.len(), 2);
        assert_eq!(ts.min_timestamp(), Some(100));
        assert_eq!(ts.max_timestamp(), Some(160));
        
        // Simuler une bougie plus ancienne (t=100) arrivant après (out-of-order)
        // Cela simule le cas où une requête HTTP plus ancienne se termine après une plus récente
        ts.update_or_append_candle(Candle::new(100, 100.5, 105.5, 99.5, 104.5)).unwrap();
        
        // Vérifier que l'ordre chronologique est maintenu
        assert_eq!(ts.len(), 2); // Ne doit pas créer de doublon
        assert_eq!(ts.min_timestamp(), Some(100));
        assert_eq!(ts.max_timestamp(), Some(160));
        
        // Vérifier que la bougie à t=100 a été mise à jour (pas dupliquée)
        let candles: Vec<i64> = ts.candles.iter().map(|c| c.timestamp).collect();
        assert_eq!(candles, vec![100, 160]);
        
        // Vérifier que la bougie à t=100 a bien été mise à jour avec les nouvelles valeurs
        let first_candle = ts.candles.first().unwrap();
        assert_eq!(first_candle.open, 100.5);
        assert_eq!(first_candle.close, 104.5);
    }

    #[test]
    fn test_update_or_append_out_of_order_middle() {
        // Test avec une bougie arrivant dans le désordre au milieu de la série
        let mut ts = TimeSeries::new();
        
        // Créer une série: t=100, t=200, t=300
        ts.push(Candle::new(100, 100.0, 105.0, 99.0, 104.0)).unwrap();
        ts.push(Candle::new(200, 104.0, 106.0, 103.0, 105.0)).unwrap();
        ts.push(Candle::new(300, 105.0, 107.0, 104.0, 106.0)).unwrap();
        
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.max_timestamp(), Some(300));
        
        // Simuler une bougie à t=150 arrivant après t=300 (out-of-order)
        ts.update_or_append_candle(Candle::new(150, 103.5, 105.5, 102.5, 104.5)).unwrap();
        
        // Vérifier que l'ordre chronologique est maintenu
        assert_eq!(ts.len(), 4);
        let candles: Vec<i64> = ts.candles.iter().map(|c| c.timestamp).collect();
        assert_eq!(candles, vec![100, 150, 200, 300]);
    }
}

