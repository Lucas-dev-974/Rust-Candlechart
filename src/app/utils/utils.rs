//! Utilitaires pour la conversion d'intervalles de temps

/// Convertit un intervalle en secondes
pub fn interval_to_seconds(interval: &str) -> i64 {
    match interval {
        "1m" => 60,
        "3m" => 180,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "2h" => 7200,
        "4h" => 14400,
        "6h" => 21600,
        "8h" => 28800,
        "12h" => 43200,
        "1d" => 86400,
        "3d" => 259200,
        "1w" => 604800,
        "1M" => 2592000, // Approximation (30 jours)
        _ => 3600, // Défaut: 1h
    }
}

/// Calcule le timestamp pour récupérer N bougies selon l'intervalle
/// 
/// Cette fonction est équivalente à `interval_to_seconds(interval) * count`
#[inline]
pub fn calculate_candles_back_timestamp(interval: &str, count: usize) -> i64 {
    interval_to_seconds(interval) * count as i64
}

/// Calcule le nombre attendu de bougies pour une période donnée selon l'intervalle
/// 
/// # Arguments
/// * `interval` - L'intervalle de la série (ex: "1h", "15m", "1d")
/// * `period_seconds` - La période en secondes (ex: 1 mois = 30 * 24 * 3600)
/// 
/// # Returns
/// Le nombre attendu de bougies pour cette période
/// 
/// # Example
/// ```
/// // Pour 1 mois (30 jours) avec un intervalle de 1h
/// let expected = calculate_expected_candles("1h", 30 * 24 * 3600);
/// // Résultat: 720 bougies (30 jours * 24 heures)
/// ```
pub fn calculate_expected_candles(interval: &str, period_seconds: i64) -> usize {
    let interval_seconds = interval_to_seconds(interval);
    if interval_seconds == 0 {
        return 0;
    }
    (period_seconds / interval_seconds) as usize
}

/// Calcule le prochain timestamp selon l'intervalle
/// 
/// # Arguments
/// * `current_timestamp` - Le timestamp actuel
/// * `interval` - L'intervalle de la série (ex: "1h", "15m", "1d")
/// 
/// # Returns
/// Le prochain timestamp (current_timestamp + interval_seconds)
pub fn next_timestamp(current_timestamp: i64, interval: &str) -> i64 {
    current_timestamp + interval_to_seconds(interval)
}

/// Trouve l'index d'une bougie avec un timestamp donné ou supérieur
/// Utilise une recherche binaire pour efficacité
/// 
/// # Arguments
/// * `candles` - La liste de bougies triée par timestamp
/// * `timestamp` - Le timestamp recherché
/// 
/// # Returns
/// L'index de la première bougie avec timestamp >= timestamp, ou None si aucune bougie trouvée
pub fn find_candle_index_by_timestamp(
    candles: &[crate::finance_chart::core::Candle],
    timestamp: i64,
) -> Option<usize> {
    if candles.is_empty() {
        return None;
    }
    
    // Recherche binaire pour trouver la première bougie avec timestamp >= timestamp
    match candles.binary_search_by_key(&timestamp, |c| c.timestamp) {
        Ok(idx) => Some(idx), // Timestamp exact trouvé
        Err(idx) => {
            // Timestamp non trouvé, idx est la position où il devrait être inséré
            if idx < candles.len() {
                Some(idx) // Retourner l'index de la première bougie >= timestamp
            } else {
                None // Toutes les bougies sont avant ce timestamp
            }
        }
    }
}

/// Trouve une bougie par timestamp (exact ou la plus proche >=)
/// 
/// # Arguments
/// * `candles` - La liste de bougies triée par timestamp
/// * `timestamp` - Le timestamp recherché
/// 
/// # Returns
/// Une référence vers la bougie trouvée, ou None si aucune bougie trouvée
pub fn find_candle_by_timestamp(
    candles: &[crate::finance_chart::core::Candle],
    timestamp: i64,
) -> Option<&crate::finance_chart::core::Candle> {
    find_candle_index_by_timestamp(candles, timestamp)
        .and_then(|idx| candles.get(idx))
}

