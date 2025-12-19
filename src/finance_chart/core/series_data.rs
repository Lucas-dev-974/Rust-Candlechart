//! Structure pour gérer plusieurs séries temporelles avec identification

use super::{Candle, TimeSeries};
use std::collections::HashMap;

/// Identifiant unique d'une série temporelle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeriesId {
    /// Nom de la série (ex: "BTCUSDT_1h")
    pub name: String,
}

impl SeriesId {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Données d'une série temporelle avec métadonnées
#[derive(Debug, Clone)]
pub struct SeriesData {
    /// Identifiant de la série
    pub id: SeriesId,
    /// Données de la série
    pub data: TimeSeries,
    /// Symbole (ex: "BTCUSDT")
    pub symbol: String,
    /// Intervalle (ex: "1h", "15m", "1d")
    pub interval: String,
    /// Couleur personnalisée pour cette série (optionnel)
    pub color: Option<iced::Color>,
}

impl SeriesData {
    /// Crée une nouvelle série avec un nom et des données
    pub fn new(id: SeriesId, symbol: String, interval: String, data: TimeSeries) -> Self {
        Self {
            id,
            data,
            symbol,
            interval,
            color: None,
        }
    }

    /// Retourne le nom complet de la série (symbol_interval)
    pub fn full_name(&self) -> String {
        format!("{}_{}", self.symbol, self.interval)
    }
}

/// Gestionnaire de plusieurs séries temporelles
#[derive(Debug, Clone)]
pub struct SeriesManager {
    /// Toutes les séries disponibles
    series: HashMap<SeriesId, SeriesData>,
    /// Séries actuellement actives (affichées)
    active_series: Vec<SeriesId>,
}

impl SeriesManager {
    /// Crée un nouveau gestionnaire de séries
    pub fn new() -> Self {
        Self {
            series: HashMap::new(),
            active_series: Vec::new(),
        }
    }

    /// Ajoute une série
    pub fn add_series(&mut self, series: SeriesData) {
        let id = series.id.clone();
        self.series.insert(id.clone(), series);
        // Activer automatiquement la première série ajoutée
        if self.active_series.is_empty() {
            self.active_series.push(id);
        }
    }

    /// Retourne une référence à une série
    pub fn get_series(&self, id: &SeriesId) -> Option<&SeriesData> {
        self.series.get(id)
    }

    /// Retourne toutes les séries disponibles
    pub fn all_series(&self) -> impl Iterator<Item = &SeriesData> {
        self.series.values()
    }

    /// Retourne les séries actives
    pub fn active_series(&self) -> impl Iterator<Item = &SeriesData> {
        self.active_series
            .iter()
            .filter_map(|id| self.series.get(id))
    }

    /// Active uniquement une série (désactive toutes les autres)
    pub fn activate_only_series(&mut self, id: SeriesId) {
        if self.series.contains_key(&id) {
            // Désactiver toutes les séries
            self.active_series.clear();
            // Activer uniquement la série sélectionnée
            self.active_series.push(id);
        }
    }

    /// Retourne toutes les bougies visibles de toutes les séries actives dans une plage temporelle
    pub fn visible_candles(&self, time_range: std::ops::Range<i64>) -> Vec<(SeriesId, &[Candle])> {
        self.active_series()
            .map(|series| {
                let candles = series.data.visible_candles(time_range.clone());
                (series.id.clone(), candles)
            })
            .collect()
    }

    /// Retourne le nombre total de séries
    pub fn total_count(&self) -> usize {
        self.series.len()
    }
}

impl Default for SeriesManager {
    fn default() -> Self {
        Self::new()
    }
}

