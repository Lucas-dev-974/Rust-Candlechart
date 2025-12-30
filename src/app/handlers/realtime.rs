//! Handlers pour la gestion des téléchargements et mises à jour temps réel

use iced::Task;
use crate::app::app_state::ChartApp;
use crate::finance_chart::core::{SeriesId, Candle};
use std::collections::HashSet;

/// Gère le chargement de l'historique complet
pub fn handle_load_full_history_complete(
    app: &mut ChartApp,
    series_id: SeriesId,
    series_name: String,
    result: Result<Vec<Candle>, String>
) -> Task<crate::app::messages::Message> {
    match result {
        Ok(candles) => {
            println!("✅ Historique complet chargé pour {}: {} bougies", series_name, candles.len());
            // Fusionner les bougies dans la série
            match app.chart_state.merge_candles(&series_id, candles) {
                crate::finance_chart::UpdateResult::MultipleCandlesAdded(count) => {
                    println!("  ✅ {} nouvelles bougies ajoutées", count);
                    // Mettre à jour le viewport pour afficher toutes les données
                    app.chart_state.update_viewport_from_series();
                    // Sauvegarder la série mise à jour de manière asynchrone
                    let mut updated_series = HashSet::new();
                    updated_series.insert(series_id);
                    return crate::app::realtime::save_series_async(app, updated_series);
                }
                crate::finance_chart::UpdateResult::Error(e) => {
                    eprintln!("  ❌ Erreur lors de la fusion: {}", e);
                }
                _ => {}
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur lors du chargement de l'historique pour {}: {}", series_name, e);
        }
    }
    Task::none()
}

/// Gère le démarrage d'un téléchargement par batch
pub fn handle_start_batch_download(
    app: &mut ChartApp,
    series_id: SeriesId,
    gaps: Vec<(i64, i64)>,
    estimated_total: usize
) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    use crate::app::app_state::DownloadProgress;
    
    if gaps.is_empty() {
        return Task::done(Message::DownloadComplete(series_id));
    }
    
    // Initialiser l'état de progression dans le gestionnaire
    let (first_start, first_end) = gaps[0];
    let progress = DownloadProgress {
        series_id: series_id.clone(),
        current_count: 0,
        estimated_total,
        current_start: first_start,
        target_end: first_end,
        gaps_remaining: gaps[1..].to_vec(),
        paused: false,
    };
    app.download_manager.start_download(progress);
    
    println!("📥 Démarrage téléchargement: {} gap(s) à combler", gaps.len());
    
    // Lancer le premier batch
    crate::app::realtime::download_batch(app, &series_id)
}

/// Gère le résultat d'un batch de téléchargement
pub fn handle_batch_download_result(
    app: &mut ChartApp,
    series_id: SeriesId,
    candles: Vec<Candle>,
    count: usize,
    _estimated: usize,
    next_end: i64
) -> Task<crate::app::messages::Message> {
    use crate::app::messages::Message;
    
    // Vérifier si le téléchargement est toujours actif dans le gestionnaire
    if !app.download_manager.is_downloading(&series_id) {
        println!("  ⚠️ Téléchargement ignoré: téléchargement annulé ou terminé pour {}", series_id.name);
        return Task::none();
    }
    
    // 1. Fusionner les nouvelles bougies immédiatement dans le graphique
    // Sans modifier le viewport pour ne pas perturber l'utilisateur
    let mut should_save = false;
    if !candles.is_empty() {
        match app.chart_state.merge_candles(&series_id, candles) {
            crate::finance_chart::UpdateResult::MultipleCandlesAdded(added) => {
                println!("  📊 +{} bougies fusionnées (total téléchargé: {})", added, count);
                // Sauvegarder seulement tous les 10 batches pour éviter les freezes
                // ou si c'est le dernier batch
                if let Some(ref progress) = app.download_manager.get_progress(&series_id) {
                    let batch_number = (progress.current_count / 1000) + 1;
                    should_save = batch_number % 10 == 0 || progress.gaps_remaining.is_empty();
                }
            }
            _ => {}
        }
    }
    
    // 2. Préparer la sauvegarde si nécessaire
    let save_task = if should_save {
        let mut updated_series = HashSet::new();
        updated_series.insert(series_id.clone());
        Some(crate::app::realtime::save_series_async(app, updated_series))
    } else {
        None
    };
    
    // 3. Mettre à jour l'état de progression et continuer
    // On télécharge du récent vers l'ancien: target_end descend vers current_start
    if app.download_manager.update_progress(&series_id, count, next_end) {
        // Vérifier si le gap actuel est terminé (on a atteint le début du gap)
        if let Some(progress) = app.download_manager.get_progress(&series_id) {
            if next_end <= progress.current_start {
                // Gap terminé, passer au suivant
                if let Some((gap_start, gap_end)) = app.download_manager.next_gap(&series_id) {
                    println!("  📥 Gap suivant: {} -> {} ({} restants)", 
                        gap_start, gap_end, 
                        app.download_manager.get_progress(&series_id)
                            .map(|p| p.gaps_remaining.len())
                            .unwrap_or(0));
                } else {
                    // Tous les gaps sont terminés!
                    println!("  🏁 Tous les gaps traités, envoi DownloadComplete");
                    // Si on doit sauvegarder, combiner avec DownloadComplete
                    if let Some(save) = save_task {
                        return Task::batch(vec![
                            save,
                            Task::done(Message::DownloadComplete(series_id))
                        ]);
                    }
                    return Task::done(Message::DownloadComplete(series_id));
                }
            }
        }
        
        // Continuer le téléchargement (en parallèle avec la sauvegarde si nécessaire)
        // Vérifier que le téléchargement n'est pas en pause avant de continuer
        if !app.download_manager.is_paused(&series_id) {
            let download_task = crate::app::realtime::download_batch(app, &series_id);
            if let Some(save) = save_task {
                return Task::batch(vec![save, download_task]);
            }
            return download_task;
        } else {
            println!("  ⏸️ Téléchargement en pause pour {}, arrêt de la chaîne", series_id.name);
        }
    }
    Task::none()
}

/// Gère la fin d'un téléchargement
pub fn handle_download_complete(app: &mut ChartApp, series_id: SeriesId) -> Task<crate::app::messages::Message> {
    println!("✅ Téléchargement terminé pour {}", series_id.name);
    
    // Retirer le téléchargement du gestionnaire
    app.download_manager.finish_download(&series_id);
    
    // Mettre à jour le viewport final
    app.chart_state.update_viewport_from_series();
    
    // Sauvegarder la série mise à jour (sauvegarde finale)
    let mut updated_series = HashSet::new();
    updated_series.insert(series_id);
    crate::app::realtime::save_series_async(app, updated_series)
}

/// Gère la pause d'un téléchargement
pub fn handle_pause_download(app: &mut ChartApp, series_id: SeriesId) -> Task<crate::app::messages::Message> {
    if app.download_manager.pause_download(&series_id) {
        println!("⏸️ Téléchargement mis en pause pour {}", series_id.name);
    }
    Task::none()
}

/// Gère la reprise d'un téléchargement
pub fn handle_resume_download(app: &mut ChartApp, series_id: SeriesId) -> Task<crate::app::messages::Message> {
    if app.download_manager.resume_download(&series_id) {
        println!("▶️ Téléchargement repris pour {}", series_id.name);
        // Relancer le téléchargement si nécessaire
        if let Some(progress) = app.download_manager.get_progress(&series_id) {
            // Vérifier si on doit continuer le téléchargement
            if !progress.gaps_remaining.is_empty() || progress.target_end > progress.current_start {
                return crate::app::realtime::download_batch(app, &series_id);
            }
        }
    }
    Task::none()
}

/// Gère l'arrêt d'un téléchargement
pub fn handle_stop_download(app: &mut ChartApp, series_id: SeriesId) -> Task<crate::app::messages::Message> {
    if app.download_manager.stop_download(&series_id) {
        println!("⏹️ Téléchargement arrêté pour {}", series_id.name);
    }
    Task::none()
}

/// Gère la sauvegarde des séries complétée
pub fn handle_save_series_complete(
    app: &mut ChartApp,
    results: Vec<(String, Result<(), String>)>
) -> Task<crate::app::messages::Message> {
    for (series_name, result) in results {
        match result {
            Ok(()) => {
                println!("  ✅ {}: Sauvegardé avec succès", series_name);
            }
            Err(e) => {
                eprintln!("  ❌ {}: Erreur lors de la sauvegarde - {}", series_name, e);
            }
        }
    }
    println!("✅ Sauvegarde des séries terminée");
    Task::none()
}

/// Gère les mises à jour temps réel
pub fn handle_realtime_update(app: &mut ChartApp) -> Task<crate::app::messages::Message> {
    app.update_realtime()
}

/// Gère les résultats des mises à jour temps réel
pub fn handle_realtime_update_complete(
    app: &mut ChartApp,
    results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>
) -> Task<crate::app::messages::Message> {
    println!("📥 RealtimeUpdateComplete: {} résultats reçus", results.len());
    app.apply_realtime_updates(results);
    Task::none()
}

/// Gère la complétion des données manquantes
pub fn handle_complete_missing_data_complete(
    app: &mut ChartApp,
    results: Vec<(SeriesId, String, Result<Vec<Candle>, String>)>
) -> Task<crate::app::messages::Message> {
    println!("📥 CompleteMissingDataComplete: {} résultats reçus", results.len());
    app.apply_complete_missing_data_results(results)
}

/// Gère la complétion des gaps
pub fn handle_complete_gaps_complete(
    app: &mut ChartApp,
    results: Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>
) -> Task<crate::app::messages::Message> {
    println!("📥 CompleteGapsComplete: {} résultats reçus", results.len());
    app.apply_complete_gaps_results(results)
}

