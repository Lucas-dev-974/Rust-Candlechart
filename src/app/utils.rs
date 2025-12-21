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

