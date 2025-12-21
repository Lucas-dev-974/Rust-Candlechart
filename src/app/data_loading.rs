//! Gestion du chargement asynchrone des séries de données
//!
//! Ce module gère le chargement des séries depuis les fichiers JSON de manière asynchrone
//! pour ne pas bloquer l'interface utilisateur au démarrage.

use iced::Task;
use crate::finance_chart::{load_all_from_directory, load_from_json};
use crate::app::{constants::DATA_FILE, messages::Message};

/// Crée une Task pour charger les séries de manière asynchrone
pub fn create_load_series_task() -> Task<Message> {
    Task::perform(
        async {
            // Charger les séries dans un thread dédié pour ne pas bloquer l'UI
            tokio::task::spawn_blocking(move || {
                match load_all_from_directory("data") {
                    Ok(series_list) => {
                        println!("✅ {} série(s) trouvée(s) dans le dossier data", series_list.len());
                        Ok(series_list)
                    }
                    Err(e) => {
                        eprintln!("❌ Erreur lors du chargement des séries depuis 'data': {}", e);
                        eprintln!("   Tentative de chargement du fichier par défaut: {}", DATA_FILE);
                        // Fallback: essayer de charger le fichier par défaut
                        match load_from_json(DATA_FILE) {
                            Ok(series) => {
                                println!("✅ Série chargée: {} bougies", series.data.len());
                                Ok(vec![series])
                            }
                            Err(e2) => {
                                eprintln!("❌ Erreur de chargement: {}", e2);
                                Err(format!("Erreur: {}", e2))
                            }
                        }
                    }
                }
            })
            .await
            .unwrap_or_else(|e| Err(format!("Erreur de thread: {}", e)))
        },
        Message::LoadSeriesFromDirectoryComplete,
    )
}

