//! Fonctions utilitaires pures pour le module realtime
//!
//! Ce module contient la logique pure (fonctions sans effets de bord)
//! extraite de realtime.rs pour faciliter les tests et la réutilisation.

use crate::app::utils::utils::calculate_candles_back_timestamp;

/// Vérifie si le nom de série est au format Binance (SYMBOL_INTERVAL)
/// 
/// # Arguments
/// * `series_name` - Le nom de la série à valider
/// 
/// # Returns
/// `true` si le format est valide (ex: "BTCUSDT_1h"), `false` sinon
/// 
/// # Examples
/// ```
/// assert!(is_binance_format("BTCUSDT_1h"));
/// assert!(!is_binance_format("invalid"));
/// ```
#[inline]
pub fn is_binance_format(series_name: &str) -> bool {
    // Validation optimisée : vérifie directement sans allocation
    if let Some(underscore_pos) = series_name.find('_') {
        underscore_pos > 0 
            && underscore_pos < series_name.len() - 1
            && series_name[underscore_pos + 1..].find('_').is_none()
    } else {
        false
    }
}

/// Extrait l'intervalle depuis un nom de série au format Binance
/// 
/// # Arguments
/// * `series_name` - Le nom de la série (format: SYMBOL_INTERVAL)
/// 
/// # Returns
/// L'intervalle extrait ou "1h" par défaut
pub fn extract_interval(series_name: &str) -> &str {
    series_name.split('_').last().unwrap_or("1h")
}

/// Détermine le timestamp depuis lequel récupérer les données
/// 
/// Si les données sont récentes (moins de 2 intervalles), on complète depuis le dernier timestamp.
/// Sinon, on récupère les 100 dernières bougies.
/// 
/// # Arguments
/// * `last_timestamp` - Le dernier timestamp connu
/// * `current_time` - Le timestamp actuel
/// * `interval` - L'intervalle de temps (ex: "1h", "15m")
/// 
/// # Returns
/// Un tuple (since_timestamp, is_stale) où:
/// - `since_timestamp`: Le timestamp depuis lequel récupérer
/// - `is_stale`: `true` si les données sont considérées comme anciennes
pub fn compute_fetch_since(last_timestamp: i64, current_time: i64, interval: &str) -> (i64, bool) {
    // Calculer le seuil pour déterminer si les données sont récentes (2 intervalles)
    let threshold_seconds = calculate_candles_back_timestamp(interval, 2);
    
    // Si les données sont récentes (moins de 2 intervalles), on complète
    if current_time - last_timestamp < threshold_seconds {
        (last_timestamp, false)
    } else {
        // Si les données sont anciennes, on récupère les 100 dernières bougies
        let since = current_time - calculate_candles_back_timestamp(interval, 100);
        (since, true)
    }
}

/// Calcule le seuil pour détecter un gap récent
/// 
/// Le seuil est basé sur l'intervalle : intervalle + 10% de buffer, minimum 5 minutes.
/// Utilisé pour déterminer si les données sont trop anciennes par rapport à maintenant.
/// 
/// # Arguments
/// * `interval_seconds` - L'intervalle en secondes
/// 
/// # Returns
/// Le seuil en secondes
#[inline]
pub fn calculate_recent_gap_threshold(interval_seconds: i64) -> i64 {
    std::cmp::max(interval_seconds + interval_seconds / 10, 300)
}

/// Obtient le timestamp Unix actuel en secondes
/// 
/// # Returns
/// Le timestamp Unix actuel (secondes depuis l'epoch)
#[inline]
pub fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Erreur horloge système")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binance_format_valid() {
        assert!(is_binance_format("BTCUSDT_1h"));
        assert!(is_binance_format("ETHUSDT_15m"));
        assert!(is_binance_format("BNB_1d"));
    }

    #[test]
    fn test_is_binance_format_invalid() {
        assert!(!is_binance_format("invalid"));
        assert!(!is_binance_format("_1h"));
        assert!(!is_binance_format("BTCUSDT_"));
        assert!(!is_binance_format("BTC_USDT_1h"));
        assert!(!is_binance_format(""));
    }

    #[test]
    fn test_extract_interval() {
        assert_eq!(extract_interval("BTCUSDT_1h"), "1h");
        assert_eq!(extract_interval("ETHUSDT_15m"), "15m");
        assert_eq!(extract_interval("invalid"), "invalid");
    }

    #[test]
    fn test_compute_fetch_since_recent() {
        let now = 1000000;
        let last = now - 1800; // 30 minutes ago
        let (since, is_stale) = compute_fetch_since(last, now, "1h");
        assert_eq!(since, last);
        assert!(!is_stale);
    }

    #[test]
    fn test_compute_fetch_since_stale() {
        let now = 1000000;
        let last = now - 100000; // Long time ago
        let (since, is_stale) = compute_fetch_since(last, now, "1h");
        assert!(is_stale);
        assert!(since < last); // Should fetch from further back
    }
}

