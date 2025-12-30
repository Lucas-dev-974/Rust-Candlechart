//! Module pour les stratégies de trading automatisées
//!
//! Ce module permet de créer et gérer des algorithmes de trading automatisés
//! qui peuvent analyser le marché et générer des signaux d'achat/vente.

pub mod strategy;
pub mod manager;
pub mod examples;

pub use strategy::*;
pub use manager::*;


