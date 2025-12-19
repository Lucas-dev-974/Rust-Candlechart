//! Module de chargement des données financières depuis des fichiers JSON
//!
//! Supporte le format Binance klines avec timestamps en millisecondes.

use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::core::{Candle, TimeSeries, SeriesData, SeriesId};

/// Structure JSON pour une bougie Binance
#[derive(Debug, Deserialize)]
struct JsonKline {
    /// Timestamp d'ouverture en millisecondes
    open_time: i64,
    /// Prix d'ouverture
    open: f64,
    /// Prix le plus haut
    high: f64,
    /// Prix le plus bas
    low: f64,
    /// Prix de clôture
    close: f64,
    /// Volume (non utilisé mais présent dans le JSON)
    #[serde(rename = "volume")]
    _volume: f64,
}

/// Structure JSON pour le fichier complet
#[derive(Debug, Deserialize)]
struct JsonData {
    /// Symbole de la paire
    symbol: String,
    /// Intervalle temporel
    interval: String,
    /// Liste des bougies
    klines: Vec<JsonKline>,
}

/// Erreur de chargement des données
#[derive(Debug)]
pub enum LoadError {
    /// Erreur d'ouverture du fichier
    FileOpen(std::io::Error),
    /// Erreur de parsing JSON
    JsonParse(serde_json::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileOpen(e) => write!(f, "Erreur d'ouverture du fichier: {}", e),
            LoadError::JsonParse(e) => write!(f, "Erreur de parsing JSON: {}", e),
        }
    }
}

impl std::error::Error for LoadError {}

/// Charge les données depuis un fichier JSON au format Binance
///
/// # Arguments
/// * `path` - Chemin vers le fichier JSON
///
/// # Returns
/// * `Ok(SeriesData)` - Série temporelle chargée avec métadonnées
/// * `Err(LoadError)` - Erreur de chargement
///
/// # Example
/// ```ignore
/// let series = load_from_json("data/BTCUSDT_1h.json")?;
/// ```
pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<SeriesData, LoadError> {
    // Ouvrir le fichier
    let file = File::open(&path).map_err(LoadError::FileOpen)?;
    let reader = BufReader::new(file);

    // Parser le JSON
    let json_data: JsonData = serde_json::from_reader(reader).map_err(LoadError::JsonParse)?;

    // Extraire le nom de la série depuis le nom du fichier
    let file_name = path.as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Créer l'ID de la série
    let series_id = SeriesId::new(file_name.clone());

    // Convertir en TimeSeries
    let mut timeseries = TimeSeries::new();

    for kline in json_data.klines {
        // Convertir timestamp millisecondes → secondes
        let timestamp = kline.open_time / 1000;

        let candle = Candle::new(
            timestamp,
            kline.open,
            kline.high,
            kline.low,
            kline.close,
        );

        timeseries.push(candle);
    }

    // Créer SeriesData avec les métadonnées
    let series = SeriesData::new(
        series_id,
        json_data.symbol,
        json_data.interval,
        timeseries,
    );

    Ok(series)
}

/// Charge toutes les séries depuis un dossier
///
/// # Arguments
/// * `dir_path` - Chemin vers le dossier contenant les fichiers JSON
///
/// # Returns
/// * `Ok(Vec<SeriesData>)` - Liste de toutes les séries chargées
/// * `Err(LoadError)` - Erreur de chargement
pub fn load_all_from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Vec<SeriesData>, LoadError> {
    use std::fs;
    
    let mut series_list = Vec::new();
    let dir = fs::read_dir(dir_path).map_err(|e| LoadError::FileOpen(e))?;
    
    for entry in dir {
        let entry = entry.map_err(|e| LoadError::FileOpen(e))?;
        let path = entry.path();
        
        // Charger uniquement les fichiers .json
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match load_from_json(&path) {
                Ok(series) => series_list.push(series),
                Err(e) => {
                    eprintln!("⚠️ Erreur lors du chargement de {:?}: {}", path, e);
                }
            }
        }
    }
    
    Ok(series_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_from_json("nonexistent.json");
        assert!(result.is_err());
    }
}

