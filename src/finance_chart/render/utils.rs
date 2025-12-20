//! Utilitaires de rendu partagés

/// Formate un prix pour l'affichage dans les tooltips et labels détaillés
/// 
/// Utilise une précision adaptative selon la valeur du prix :
/// - >= 100 : 2 décimales
/// - >= 1 : 4 décimales  
/// - < 1 : 6 décimales
pub fn format_price_detailed(price: f64) -> String {
    if price >= 100.0 {
        format!("{:.2}", price)
    } else if price >= 1.0 {
        format!("{:.4}", price)
    } else {
        format!("{:.6}", price)
    }
}

/// Formate un prix pour l'affichage dans les labels compacts (crosshair, axes)
/// 
/// Utilise une précision adaptative selon la valeur du prix :
/// - >= 10000 : 0 décimales
/// - >= 100 : 1 décimale
/// - >= 1 : 2 décimales
/// - < 1 : 4 décimales
pub fn format_price_compact(price: f64) -> String {
    if price >= 10000.0 {
        format!("{:.0}", price)
    } else if price >= 100.0 {
        format!("{:.1}", price)
    } else if price >= 1.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    }
}

/// Formate un prix pour l'affichage dans les badges (lignes horizontales)
/// 
/// Utilise une précision adaptative selon la valeur du prix :
/// - >= 10000 : 0 décimales
/// - >= 100 : 1 décimale
/// - < 100 : 2 décimales
pub fn format_price_badge(price: f64) -> String {
    if price >= 10000.0 {
        format!("{:.0}", price)
    } else if price >= 100.0 {
        format!("{:.1}", price)
    } else {
        format!("{:.2}", price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price_detailed() {
        assert_eq!(format_price_detailed(50000.0), "50000.00");
        assert_eq!(format_price_detailed(100.5), "100.50");
        assert_eq!(format_price_detailed(50.1234), "50.1234");
        assert_eq!(format_price_detailed(0.123456), "0.123456");
    }

    #[test]
    fn test_format_price_compact() {
        assert_eq!(format_price_compact(50000.0), "50000");
        assert_eq!(format_price_compact(100.5), "100.5");
        assert_eq!(format_price_compact(50.1234), "50.12");
        assert_eq!(format_price_compact(0.123456), "0.1235");
    }

    #[test]
    fn test_format_price_badge() {
        assert_eq!(format_price_badge(50000.0), "50000");
        assert_eq!(format_price_badge(100.5), "100.5");
        assert_eq!(format_price_badge(50.1234), "50.12");
    }
}
