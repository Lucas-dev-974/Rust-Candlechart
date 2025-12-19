//! Exemple d'utilisation simplifiée du crate CandleChart
//!
//! Cet exemple montre comment utiliser l'API simplifiée en seulement 10-15 lignes.

use candlechart::simple_app;

fn main() -> iced::Result {
    // Utilisation ultra-simplifiée : une seule ligne !
    simple_app("../data", 1200.0, 800.0)
}

