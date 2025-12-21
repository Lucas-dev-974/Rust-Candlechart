//! Module de chargement et sauvegarde des données financières depuis/vers des fichiers JSON
//!
//! Supporte le format Binance klines avec timestamps en millisecondes.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::core::{Candle, TimeSeries, SeriesData, SeriesId};

/// Structure JSON pour une bougie Binance
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
    /// Erreur de validation des données
    Validation(String),
    /// Fichier trop volumineux
    FileTooLarge { size: u64, max_size: u64 },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileOpen(e) => write!(f, "Erreur d'ouverture du fichier: {}", e),
            LoadError::JsonParse(e) => write!(f, "Erreur de parsing JSON: {}", e),
            LoadError::Validation(msg) => write!(f, "Erreur de validation: {}", msg),
            LoadError::FileTooLarge { size, max_size } => {
                write!(f, "Fichier trop volumineux: {} bytes (max: {} bytes)", size, max_size)
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// Taille maximale d'un fichier JSON (100 MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Nombre maximum de bougies par fichier
const MAX_CANDLES: usize = 1_000_000;

/// Valide les données JSON chargées
fn validate_json_data(data: &JsonData) -> Result<(), LoadError> {
    // Vérifier que le symbole n'est pas vide
    if data.symbol.trim().is_empty() {
        return Err(LoadError::Validation("Le symbole ne peut pas être vide".to_string()));
    }
    
    // Vérifier que l'intervalle n'est pas vide
    if data.interval.trim().is_empty() {
        return Err(LoadError::Validation("L'intervalle ne peut pas être vide".to_string()));
    }
    
    // Vérifier le nombre de bougies
    if data.klines.len() > MAX_CANDLES {
        return Err(LoadError::Validation(format!(
            "Trop de bougies: {} (max: {})",
            data.klines.len(),
            MAX_CANDLES
        )));
    }
    
    // Valider chaque bougie
    for (idx, kline) in data.klines.iter().enumerate() {
        // Vérifier que les prix sont valides (positifs et finis)
        if !kline.open.is_finite() || kline.open <= 0.0 {
            return Err(LoadError::Validation(format!(
                "Bougie {}: prix d'ouverture invalide: {}",
                idx, kline.open
            )));
        }
        if !kline.high.is_finite() || kline.high <= 0.0 {
            return Err(LoadError::Validation(format!(
                "Bougie {}: prix maximum invalide: {}",
                idx, kline.high
            )));
        }
        if !kline.low.is_finite() || kline.low <= 0.0 {
            return Err(LoadError::Validation(format!(
                "Bougie {}: prix minimum invalide: {}",
                idx, kline.low
            )));
        }
        if !kline.close.is_finite() || kline.close <= 0.0 {
            return Err(LoadError::Validation(format!(
                "Bougie {}: prix de clôture invalide: {}",
                idx, kline.close
            )));
        }
        
        // Vérifier la cohérence OHLC (high >= low, high >= open/close, low <= open/close)
        if kline.high < kline.low {
            return Err(LoadError::Validation(format!(
                "Bougie {}: high ({}) < low ({})",
                idx, kline.high, kline.low
            )));
        }
        if kline.high < kline.open || kline.high < kline.close {
            return Err(LoadError::Validation(format!(
                "Bougie {}: high ({}) doit être >= open ({}) et close ({})",
                idx, kline.high, kline.open, kline.close
            )));
        }
        if kline.low > kline.open || kline.low > kline.close {
            return Err(LoadError::Validation(format!(
                "Bougie {}: low ({}) doit être <= open ({}) et close ({})",
                idx, kline.low, kline.open, kline.close
            )));
        }
        
        // Vérifier le timestamp
        if kline.open_time <= 0 {
            return Err(LoadError::Validation(format!(
                "Bougie {}: timestamp invalide: {}",
                idx, kline.open_time
            )));
        }
    }
    
    Ok(())
}

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
    // Vérifier la taille du fichier avant de l'ouvrir
    let metadata = std::fs::metadata(&path).map_err(LoadError::FileOpen)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(LoadError::FileTooLarge {
            size: metadata.len(),
            max_size: MAX_FILE_SIZE,
        });
    }
    
    // Ouvrir le fichier
    let file = File::open(&path).map_err(LoadError::FileOpen)?;
    let reader = BufReader::new(file);

    // Parser le JSON
    let json_data: JsonData = serde_json::from_reader(reader).map_err(LoadError::JsonParse)?;
    
    // Valider les données
    validate_json_data(&json_data)?;

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

        // Ignorer les bougies invalides (déjà validées dans validate_json_data)
        if let Err(e) = timeseries.push(candle) {
            eprintln!("⚠️ Bougie invalide ignorée lors du chargement: {}", e);
        }
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

/// Erreur de sauvegarde des données
#[derive(Debug)]
pub enum SaveError {
    /// Erreur d'écriture du fichier
    FileWrite(std::io::Error),
    /// Erreur de sérialisation JSON
    JsonSerialize(serde_json::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::FileWrite(e) => write!(f, "Erreur d'écriture du fichier: {}", e),
            SaveError::JsonSerialize(e) => write!(f, "Erreur de sérialisation JSON: {}", e),
        }
    }
}

impl std::error::Error for SaveError {}

/// Sauvegarde une série dans un fichier JSON au format Binance
///
/// # Arguments
/// * `series` - La série à sauvegarder
/// * `path` - Chemin vers le fichier JSON de destination
///
/// # Returns
/// * `Ok(())` - Sauvegarde réussie
/// * `Err(SaveError)` - Erreur de sauvegarde
///
/// # Example
/// ```ignore
/// save_to_json(&series, "data/BTCUSDT_1h.json")?;
/// ```
pub fn save_to_json<P: AsRef<Path>>(series: &SeriesData, path: P) -> Result<(), SaveError> {
    // Convertir les bougies en format JSON
    let klines: Vec<JsonKline> = series.data.all_candles()
        .iter()
        .map(|candle| JsonKline {
            open_time: candle.timestamp * 1000, // Convertir secondes → millisecondes
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            _volume: 0.0, // Volume non utilisé
        })
        .collect();

    // Créer la structure JSON complète
    let json_data = JsonData {
        symbol: series.symbol.clone(),
        interval: series.interval.clone(),
        klines,
    };

    // Sérialiser en JSON avec indentation
    let json = serde_json::to_string_pretty(&json_data)
        .map_err(SaveError::JsonSerialize)?;

    // Écrire dans le fichier
    std::fs::write(&path, json)
        .map_err(SaveError::FileWrite)?;

    Ok(())
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

