//! Gestionnaire de téléchargements multiples
//!
//! Ce module gère plusieurs téléchargements simultanés de séries différentes.

use std::collections::HashMap;
use crate::finance_chart::core::SeriesId;
use crate::app::app_state::DownloadProgress;

/// Gestionnaire de téléchargements multiples
#[derive(Debug, Clone)]
pub struct DownloadManager {
    /// Tous les téléchargements en cours, indexés par SeriesId
    downloads: HashMap<SeriesId, DownloadProgress>,
}

impl DownloadManager {
    /// Crée un nouveau gestionnaire de téléchargements
    pub fn new() -> Self {
        Self {
            downloads: HashMap::new(),
        }
    }

    /// Démarre un nouveau téléchargement
    pub fn start_download(&mut self, progress: DownloadProgress) {
        let series_id = progress.series_id.clone();
        self.downloads.insert(series_id, progress);
    }

    /// Met à jour la progression d'un téléchargement
    pub fn update_progress(&mut self, series_id: &SeriesId, count: usize, next_end: i64) -> bool {
        if let Some(progress) = self.downloads.get_mut(series_id) {
            progress.current_count = count;
            progress.target_end = next_end;
            true
        } else {
            false
        }
    }

    /// Passe au gap suivant pour un téléchargement
    pub fn next_gap(&mut self, series_id: &SeriesId) -> Option<(i64, i64)> {
        if let Some(progress) = self.downloads.get_mut(series_id) {
            if progress.gaps_remaining.is_empty() {
                None
            } else {
                let (gap_start, gap_end) = progress.gaps_remaining.remove(0);
                progress.current_start = gap_start;
                progress.target_end = gap_end;
                Some((gap_start, gap_end))
            }
        } else {
            None
        }
    }

    /// Termine un téléchargement
    pub fn finish_download(&mut self, series_id: &SeriesId) -> bool {
        self.downloads.remove(series_id).is_some()
    }

    /// Récupère la progression d'un téléchargement
    pub fn get_progress(&self, series_id: &SeriesId) -> Option<&DownloadProgress> {
        self.downloads.get(series_id)
    }

    /// Récupère tous les téléchargements en cours
    pub fn all_downloads(&self) -> &HashMap<SeriesId, DownloadProgress> {
        &self.downloads
    }

    /// Vérifie si un téléchargement est en cours pour une série
    pub fn is_downloading(&self, series_id: &SeriesId) -> bool {
        self.downloads.contains_key(series_id)
    }

    /// Vérifie si un téléchargement est en pause
    pub fn is_paused(&self, series_id: &SeriesId) -> bool {
        self.downloads.get(series_id)
            .map(|p| p.paused)
            .unwrap_or(false)
    }

    /// Met en pause un téléchargement
    pub fn pause_download(&mut self, series_id: &SeriesId) -> bool {
        if let Some(progress) = self.downloads.get_mut(series_id) {
            progress.paused = true;
            true
        } else {
            false
        }
    }

    /// Reprend un téléchargement en pause
    pub fn resume_download(&mut self, series_id: &SeriesId) -> bool {
        if let Some(progress) = self.downloads.get_mut(series_id) {
            progress.paused = false;
            true
        } else {
            false
        }
    }

    /// Arrête un téléchargement (le supprime du gestionnaire)
    pub fn stop_download(&mut self, series_id: &SeriesId) -> bool {
        self.downloads.remove(series_id).is_some()
    }

    /// Retourne le nombre de téléchargements en cours
    pub fn count(&self) -> usize {
        self.downloads.len()
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

