mod finance_chart;
mod app;

use iced::{Task, Size, window, exit, Element};
use std::sync::Arc;
use std::collections::HashSet;
use finance_chart::{
    YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage,
    BinanceProvider,
    core::{SeriesId, Candle},
    ProviderType,
    settings::color_fields,
};

// Utiliser les constantes du module app::constants
use app::constants::*;

fn main() -> iced::Result {
    iced::daemon(ChartApp::new, ChartApp::update, ChartApp::view)
        .title(ChartApp::title)
        .theme(ChartApp::theme)
        .subscription(ChartApp::subscription)
        .run()
}

// Utiliser ChartApp et Message du module app
use app::{ChartApp, Message, window_manager::WindowType};

impl ChartApp {

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // === Gestion des messages du graphique ===
            Message::Chart(chart_msg) => {
                crate::app::handlers::handle_chart_message(self, chart_msg);
                Task::none()
            }
            
            // === Gestion des messages des axes ===
            Message::YAxis(YAxisMessage::ZoomVertical { factor }) => {
                self.chart_state.zoom_vertical(factor);
                Task::none()
            }
            Message::XAxis(XAxisMessage::ZoomHorizontal { factor }) => {
                self.chart_state.zoom(factor);
                Task::none()
            }
            
            // === Gestion des messages du panel d'outils ===
            Message::ToolsPanel(ToolsPanelMessage::ToggleTool { tool }) => {
                if self.tools_state.selected_tool == Some(tool) {
                    self.tools_state.selected_tool = None;
                } else {
                    self.tools_state.selected_tool = Some(tool);
                }
                Task::none()
            }
            Message::ToolsPanel(ToolsPanelMessage::ToggleIndicatorsPanel) => {
                self.indicators_panel_open = !self.indicators_panel_open;
                Task::none()
            }
            
            // === Gestion des messages du panel de séries ===
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                println!("🔄 Sélection de la série: {}", series_name);
                
                // Trouver le SeriesId correspondant au nom
                let series_id_opt = self.chart_state.series_manager.all_series()
                    .find(|s| s.full_name() == series_name)
                    .map(|s| s.id.clone());
                
                if let Some(series_id) = series_id_opt {
                    // Activer uniquement cette série (désactive toutes les autres)
                    self.chart_state.series_manager.activate_only_series(series_id.clone());
                    // Mettre à jour le viewport après activation
                    self.chart_state.update_viewport_from_series();
                    
                    // Vérifier automatiquement les gaps de la série
                    // et télécharger les données manquantes (historique + gaps)
                    if let Some(series) = self.chart_state.series_manager.get_series(&series_id) {
                        let current_count = series.data.len();
                        let oldest = series.data.min_timestamp();
                        
                        println!("🔍 Vérification série {}: {} bougies", series_name, current_count);
                        if let Some(ts) = oldest {
                            println!("  📅 Première bougie: {}", ts);
                        }
                        
                        // Vérifier s'il y a des gaps à combler (récent, internes, ou historique)
                        // has_gaps_to_fill vérifie déjà si la série est vide
                        let has_gaps = crate::app::realtime::has_gaps_to_fill(self, &series_id);
                        
                        if has_gaps {
                            println!("📥 Série {} a des gaps à combler, lancement de l'auto-complétion...", series_name);
                            return crate::app::realtime::auto_complete_series(self, series_id);
                        } else {
                            println!("✅ Série {} complète ({} bougies, pas de gaps)", series_name, current_count);
                        }
                    }
                }
                Task::none()
            }
            
            // === Gestion des fenêtres ===
            Message::MainWindowOpened(_id) => Task::none(),
            
            Message::LoadSeriesFromDirectory => Task::none(),
            
            Message::LoadSeriesFromDirectoryComplete(result) => {
                match result {
                    Ok(series_list) => {
                        for series in series_list {
                            let series_name = series.full_name();
                            println!(
                                "  📊 {}: {} bougies ({} - {})",
                                series_name,
                                series.data.len(),
                                series.symbol,
                                series.interval
                            );
                            self.chart_state.add_series(series);
                        }
                        // Calculer et stocker le MACD pré-calculé une fois après le chargement initial
                        let _ = self.chart_state.compute_and_store_macd();
                        if self.chart_state.series_manager.total_count() == 0 {
                            eprintln!("⚠️ Aucune série chargée. Vérifiez que le dossier 'data' contient des fichiers JSON.");
                            return Task::none();
                        }
                        
                        // Vérifier si la série active a des gaps à combler
                        let active_series_info = self.chart_state.series_manager.active_series()
                            .next()
                            .map(|s| {
                                let oldest = s.data.min_timestamp();
                                (s.id.clone(), s.full_name(), s.data.len(), oldest)
                            });
                        
                        if let Some((series_id, series_name, candle_count, oldest)) = active_series_info {
                            println!("🔍 Vérification série active {}: {} bougies", series_name, candle_count);
                            if let Some(ts) = oldest {
                                println!("  📅 Première bougie: {}", ts);
                            }
                            
                            // Vérifier s'il y a des gaps à combler (récent, internes, ou historique)
                            // has_gaps_to_fill vérifie déjà si la série est vide
                            let has_gaps = crate::app::realtime::has_gaps_to_fill(self, &series_id);
                            
                            if has_gaps {
                                println!("📥 Série active {} a des gaps à combler, lancement de l'auto-complétion...", series_name);
                                return crate::app::realtime::auto_complete_series(self, series_id);
                            } else {
                                println!("✅ Série active {} complète ({} bougies, pas de gaps)", series_name, candle_count);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Erreur lors du chargement des séries: {}", e);
                    }
                }
                Task::none()
            }
            
            Message::OpenSettings => {
                if self.windows.is_open(WindowType::Settings) {
                    return Task::none();
                }
                self.editing_style = Some(self.chart_style.clone());
                self.editing_color_index = None;
                
                let (id, task) = window::open(window::Settings {
                    size: Size::new(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT),
                    resizable: false,
                    ..Default::default()
                });
                self.windows.set_id(WindowType::Settings, id);
                task.map(Message::SettingsWindowOpened)
            }
            
            Message::SettingsWindowOpened(_id) => Task::none(),
            
            Message::OpenDownloads => {
                if self.windows.is_open(WindowType::Downloads) {
                    return Task::none();
                }
                
                let (id, task) = window::open(window::Settings {
                    size: Size::new(500.0, 400.0),
                    resizable: true,
                    ..Default::default()
                });
                self.windows.set_id(WindowType::Downloads, id);
                task.map(Message::DownloadsWindowOpened)
            }
            
            Message::DownloadsWindowOpened(_id) => Task::none(),
            
            Message::WindowClosed(id) => {
                match self.windows.get_window_type(id) {
                    Some(WindowType::Settings) => {
                        self.windows.remove_id(WindowType::Settings);
                        self.editing_style = None;
                        self.editing_color_index = None;
                    }
                    Some(WindowType::ProviderConfig) => {
                        self.windows.remove_id(WindowType::ProviderConfig);
                        self.editing_provider_token.clear();
                    }
                    Some(WindowType::Downloads) => {
                        self.windows.remove_id(WindowType::Downloads);
                    }
                    Some(WindowType::Main) => {
                        // Quitter l'application quand la fenêtre principale est fermée
                        // exit() fermera automatiquement toutes les fenêtres
                        return exit();
                    }
                    None => {}
                }
                Task::none()
            }
            
            // === Gestion de la configuration des providers ===
            Message::OpenProviderConfig => {
                if self.windows.is_open(WindowType::ProviderConfig) {
                    return Task::none();
                }
                
                // Initialiser les tokens en cours d'édition
                for provider_type in ProviderType::all() {
                    if let Some(config) = self.provider_config.providers.get(&provider_type) {
                        self.editing_provider_token.insert(
                            provider_type,
                            config.api_token.clone().unwrap_or_default(),
                        );
                    } else {
                        self.editing_provider_token.insert(provider_type, String::new());
                    }
                }
                
                let (id, task) = window::open(window::Settings {
                    size: Size::new(600.0, 500.0),
                    resizable: false,
                    ..Default::default()
                });
                self.windows.set_id(WindowType::ProviderConfig, id);
                task.map(Message::ProviderConfigWindowOpened)
            }
            
            Message::ProviderConfigWindowOpened(_id) => Task::none(),
            
            Message::UpdateProviderToken(provider_type, token) => {
                self.editing_provider_token.insert(provider_type, token);
                Task::none()
            }
            
            Message::ApplyProviderConfig => {
                // Appliquer les tokens modifiés
                for (provider_type, token) in &self.editing_provider_token {
                    let token_opt = if token.is_empty() {
                        None
                    } else {
                        Some(token.clone())
                    };
                    self.provider_config.set_provider_token(*provider_type, token_opt);
                }
                
                // Sauvegarder la configuration
                if let Err(e) = self.provider_config.save_to_file("provider_config.json") {
                    eprintln!("⚠️ Erreur sauvegarde configuration providers: {}", e);
                } else {
                    println!("✅ Configuration des providers sauvegardée dans provider_config.json");
                }
                
                // Recréer le provider avec la nouvelle configuration (Arc pour partage efficace)
                if let Some(config) = self.provider_config.active_config() {
                    self.binance_provider = Arc::new(BinanceProvider::with_token(config.api_token.clone()));
                    println!("✅ Provider recréé avec la nouvelle configuration");
                }
                
                // Fermer la fenêtre
                if let Some(id) = self.windows.get_id(WindowType::ProviderConfig) {
                    self.windows.remove_id(WindowType::ProviderConfig);
                    self.editing_provider_token.clear();
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::SelectProvider(provider_type) => {
                self.provider_config.set_active_provider(provider_type);
                
                // Recréer le provider avec la configuration du nouveau provider actif (Arc pour partage efficace)
                if let Some(config) = self.provider_config.active_config() {
                    self.binance_provider = Arc::new(BinanceProvider::with_token(config.api_token.clone()));
                    println!("✅ Provider changé et recréé");
                }
                
                Task::none()
            }
            
            Message::CancelProviderConfig => {
                if let Some(id) = self.windows.get_id(WindowType::ProviderConfig) {
                    self.windows.remove_id(WindowType::ProviderConfig);
                    self.editing_provider_token.clear();
                    return window::close(id);
                }
                Task::none()
            }
            
            // === Gestion des settings ===
            Message::SelectColor(field_index, color) => {
                if let Some(ref mut style) = self.editing_style {
                    let fields = color_fields();
                    if field_index < fields.len() {
                        (fields[field_index].set)(style, color);
                    }
                }
                self.editing_color_index = None;
                Task::none()
            }
            
            Message::ApplySettings => {
                if let Some(new_style) = self.editing_style.take() {
                    self.chart_style = new_style.clone();
                    if let Err(e) = new_style.save_to_file("chart_style.json") {
                        eprintln!("⚠️ Erreur sauvegarde style: {}", e);
                    } else {
                        println!("✅ Style sauvegardé dans chart_style.json");
                    }
                }
                if let Some(id) = self.windows.get_id(WindowType::Settings) {
                    self.windows.remove_id(WindowType::Settings);
                    self.editing_color_index = None;
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::CancelSettings => {
                self.editing_style = None;
                self.editing_color_index = None;
                if let Some(id) = self.windows.get_id(WindowType::Settings) {
                    self.windows.remove_id(WindowType::Settings);
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::ToggleColorPicker(index) => {
                if self.editing_color_index == Some(index) {
                    self.editing_color_index = None;
                } else {
                    self.editing_color_index = Some(index);
                }
                Task::none()
            }
            
            Message::ToggleAutoScroll => {
                if let Some(ref mut style) = self.editing_style {
                    style.auto_scroll_enabled = !style.auto_scroll_enabled;
                }
                Task::none()
            }
            
            // === Messages temps réel ===
            Message::CompleteMissingData => {
                self.complete_missing_data()
            }
            
            Message::CompleteMissingDataComplete(results) => {
                println!("📥 CompleteMissingDataComplete: {} résultats reçus", results.len());
                self.apply_complete_missing_data_results(results)
            }
            
            Message::LoadFullHistory(series_id) => {
                crate::app::realtime::load_full_history(self, series_id)
            }
            
            Message::LoadFullHistoryComplete(series_id, series_name, result) => {
                
                match result {
                    Ok(candles) => {
                        println!("✅ Historique complet chargé pour {}: {} bougies", series_name, candles.len());
                        // Fusionner les bougies dans la série
                        match self.chart_state.merge_candles(&series_id, candles) {
                            crate::finance_chart::UpdateResult::MultipleCandlesAdded(count) => {
                                println!("  ✅ {} nouvelles bougies ajoutées", count);
                                // Mettre à jour le viewport pour afficher toutes les données
                                self.chart_state.update_viewport_from_series();
                                // Sauvegarder la série mise à jour de manière asynchrone
                                use std::collections::HashSet;
                                let mut updated_series = HashSet::new();
                                updated_series.insert(series_id);
                                return crate::app::realtime::save_series_async(self, updated_series);
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
            
            Message::StartBatchDownload(series_id, gaps, estimated_total) => {
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
                self.download_manager.start_download(progress);
                
                println!("📥 Démarrage téléchargement: {} gap(s) à combler", gaps.len());
                
                // Lancer le premier batch
                crate::app::realtime::download_batch(self, &series_id)
            }
            
            Message::BatchDownloadResult(series_id, candles, count, _estimated, next_end) => {
                // Vérifier si le téléchargement est toujours actif dans le gestionnaire
                if !self.download_manager.is_downloading(&series_id) {
                    println!("  ⚠️ Téléchargement ignoré: téléchargement annulé ou terminé pour {}", series_id.name);
                    return Task::none();
                }
                
                // 1. Fusionner les nouvelles bougies immédiatement dans le graphique
                // Sans modifier le viewport pour ne pas perturber l'utilisateur
                let mut should_save = false;
                if !candles.is_empty() {
                    match self.chart_state.merge_candles(&series_id, candles) {
                        crate::finance_chart::UpdateResult::MultipleCandlesAdded(added) => {
                            println!("  📊 +{} bougies fusionnées (total téléchargé: {})", added, count);
                            // Sauvegarder seulement tous les 10 batches pour éviter les freezes
                            // ou si c'est le dernier batch
                            if let Some(ref progress) = self.download_manager.get_progress(&series_id) {
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
                    Some(crate::app::realtime::save_series_async(self, updated_series))
                } else {
                    None
                };
                
                // 3. Mettre à jour l'état de progression et continuer
                // On télécharge du récent vers l'ancien: target_end descend vers current_start
                if self.download_manager.update_progress(&series_id, count, next_end) {
                    // Vérifier si le gap actuel est terminé (on a atteint le début du gap)
                    if let Some(progress) = self.download_manager.get_progress(&series_id) {
                        if next_end <= progress.current_start {
                            // Gap terminé, passer au suivant
                            if let Some((gap_start, gap_end)) = self.download_manager.next_gap(&series_id) {
                                println!("  📥 Gap suivant: {} -> {} ({} restants)", 
                                    gap_start, gap_end, 
                                    self.download_manager.get_progress(&series_id)
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
                    if !self.download_manager.is_paused(&series_id) {
                        let download_task = crate::app::realtime::download_batch(self, &series_id);
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
            
            Message::DownloadComplete(series_id) => {
                println!("✅ Téléchargement terminé pour {}", series_id.name);
                
                // Retirer le téléchargement du gestionnaire
                self.download_manager.finish_download(&series_id);
                
                // Mettre à jour le viewport final
                self.chart_state.update_viewport_from_series();
                
                // Sauvegarder la série mise à jour (sauvegarde finale)
                let mut updated_series = HashSet::new();
                updated_series.insert(series_id);
                crate::app::realtime::save_series_async(self, updated_series)
            }
            
            Message::PauseDownload(series_id) => {
                if self.download_manager.pause_download(&series_id) {
                    println!("⏸️ Téléchargement mis en pause pour {}", series_id.name);
                }
                Task::none()
            }
            
            Message::ResumeDownload(series_id) => {
                if self.download_manager.resume_download(&series_id) {
                    println!("▶️ Téléchargement repris pour {}", series_id.name);
                    // Relancer le téléchargement si nécessaire
                    if let Some(progress) = self.download_manager.get_progress(&series_id) {
                        // Vérifier si on doit continuer le téléchargement
                        if !progress.gaps_remaining.is_empty() || progress.target_end > progress.current_start {
                            return crate::app::realtime::download_batch(self, &series_id);
                        }
                    }
                }
                Task::none()
            }
            
            Message::StopDownload(series_id) => {
                if self.download_manager.stop_download(&series_id) {
                    println!("⏹️ Téléchargement arrêté pour {}", series_id.name);
                }
                Task::none()
            }
            
            Message::CompleteGaps => {
                self.complete_gaps()
            }
            
            Message::CompleteGapsComplete(results) => {
                println!("📥 CompleteGapsComplete: {} résultats reçus", results.len());
                self.apply_complete_gaps_results(results)
            }
            
            Message::SaveSeriesComplete(results) => {
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
            
            Message::RealtimeUpdate => {
                self.update_realtime()
            }
            
            Message::RealtimeUpdateComplete(results) => {
                println!("📥 RealtimeUpdateComplete: {} résultats reçus", results.len());
                self.apply_realtime_updates(results);
                Task::none()
            }
            
            // === Gestion des panneaux latéraux ===
            Message::ToggleVolumePanel => {
                self.panels.volume.toggle_visibility();
                // Sauvegarder l'état des panneaux après changement de visibilité
                self.save_panel_state();
                Task::none()
            }
            Message::ToggleRSIPanel => {
                self.panels.rsi.toggle_visibility();
                // Sauvegarder l'état des panneaux après changement de visibilité
                self.save_panel_state();
                Task::none()
            }
            Message::ToggleMACDPanel => {
                self.panels.macd.toggle_visibility();
                // Sauvegarder l'état des panneaux après changement de visibilité
                self.save_panel_state();
                Task::none()
            }
            Message::StartResizeRightPanel(pos) => {
                self.panels.right.start_resize(pos);
                Task::none()
            }
            Message::StartResizeBottomPanel(pos) => {
                self.panels.bottom.start_resize(pos);
                Task::none()
            }
            Message::UpdateResizeRightPanel(pos) => {
                self.panels.right.update_resize(pos, true);
                Task::none()
            }
            Message::UpdateResizeBottomPanel(pos) => {
                self.panels.bottom.update_resize(pos, false);
                Task::none()
            }
            Message::EndResizeRightPanel => {
                self.panels.right.end_resize();
                // Sauvegarder l'état des panneaux après redimensionnement
                self.save_panel_state();
                Task::none()
            }
            Message::EndResizeBottomPanel => {
                self.panels.bottom.end_resize();
                // Sauvegarder l'état des panneaux après redimensionnement
                self.save_panel_state();
                Task::none()
            }
            Message::StartResizeVolumePanel(pos) => {
                self.panels.volume.start_resize(pos);
                Task::none()
            }
            Message::UpdateResizeVolumePanel(pos) => {
                self.panels.volume.update_resize(pos, false);
                Task::none()
            }
            Message::EndResizeVolumePanel => {
                self.panels.volume.end_resize();
                // Sauvegarder l'état des panneaux après redimensionnement
                self.save_panel_state();
                Task::none()
            }
            Message::StartResizeRSIPanel(pos) => {
                self.panels.rsi.start_resize(pos);
                Task::none()
            }
            Message::StartResizeMACDPanel(pos) => {
                self.panels.macd.start_resize(pos);
                Task::none()
            }
            Message::UpdateResizeRSIPanel(pos) => {
                self.panels.rsi.update_resize(pos, false);
                Task::none()
            }
            Message::UpdateResizeMACDPanel(pos) => {
                self.panels.macd.update_resize(pos, false);
                Task::none()
            }
            Message::EndResizeRSIPanel => {
                self.panels.rsi.end_resize();
                // Sauvegarder l'état des panneaux après redimensionnement
                self.save_panel_state();
                Task::none()
            }
            Message::EndResizeMACDPanel => {
                self.panels.macd.end_resize();
                // Sauvegarder l'état des panneaux après redimensionnement
                self.save_panel_state();
                Task::none()
            }
            Message::StartDragSection(section) => {
                let section_is_in_right = self.bottom_panel_sections.is_section_in_right_panel(section);
                
                // Démarrer un nouveau drag
                self.dragging_section = Some(section);
                self.drag_from_right_panel = section_is_in_right;
                self.drag_over_right_panel = section_is_in_right;
                
                // Sélectionner la section dans le bon panneau
                if section_is_in_right {
                    self.bottom_panel_sections.set_active_right_section(section);
                } else {
                    self.bottom_panel_sections.set_active_section(section);
                }
                Task::none()
            }
            Message::UpdateDragPosition(position) => {
                if self.dragging_section.is_some() {
                    self.drag_position = Some(position);
                }
                Task::none()
            }
            Message::EndDragSection => {
                if let Some(section) = self.dragging_section.take() {
                    if self.drag_from_right_panel {
                        // On drague depuis le panneau de droite
                        // Toujours déplacer vers le bas (on a relâché sur le panneau du bas)
                        self.bottom_panel_sections.move_section_to_bottom_panel(section);
                        self.save_panel_state();
                    } else {
                        // On drague depuis le panneau du bas
                        if self.drag_over_right_panel {
                            // On est sur le panneau de droite → déplacer vers la droite
                            self.bottom_panel_sections.move_section_to_right_panel(section);
                            self.save_panel_state();
                        }
                        // Sinon, on reste sur le panneau du bas, ne rien faire
                    }
                }
                self.drag_from_right_panel = false;
                self.drag_over_right_panel = false;
                self.drag_position = None;
                Task::none()
            }
            Message::DragEnterRightPanel => {
                self.drag_over_right_panel = true;
                Task::none()
            }
            Message::DragExitRightPanel => {
                self.drag_over_right_panel = false;
                Task::none()
            }
            Message::SetRightPanelFocus(focused) => {
                self.panels.right.set_focused(focused);
                Task::none()
            }
            Message::SetBottomPanelFocus(focused) => {
                self.panels.bottom.set_focused(focused);
                Task::none()
            }
            Message::SetVolumePanelFocus(focused) => {
                self.panels.volume.set_focused(focused);
                Task::none()
            }
            Message::SetRSIPanelFocus(focused) => {
                self.panels.rsi.set_focused(focused);
                Task::none()
            }
            Message::SetMACDPanelFocus(focused) => {
                self.panels.macd.set_focused(focused);
                Task::none()
            }
            Message::ClearPanelFocus => {
                self.panels.right.set_focused(false);
                self.panels.bottom.set_focused(false);
                self.panels.volume.set_focused(false);
                self.panels.rsi.set_focused(false);
                Task::none()
            }
            Message::ToggleAccountType => {
                // Basculer entre démo et réel
                let new_type = if self.account_type.is_demo() {
                    crate::app::account_type::AccountType::Real
                } else {
                    crate::app::account_type::AccountType::Demo
                };
                self.account_type.set_account_type(new_type);
                Task::none()
            }
            
            Message::TestProviderConnection => {
                self.provider_connection_testing = true;
                self.provider_connection_status = None;
                crate::app::realtime::test_provider_connection(self)
            }
            
            Message::ProviderConnectionTestComplete(result) => {
                self.provider_connection_testing = false;
                self.provider_connection_status = Some(result.is_ok());
                if let Err(e) = &result {
                    eprintln!("❌ Test de connexion échoué: {}", e);
                } else {
                    println!("✅ Connexion au provider réussie");
                }
                Task::none()
            }
        }
    }
    
    /// Complète les données manquantes depuis Binance pour toutes les séries
    /// 
    /// Utilise Iced Tasks pour faire les requêtes en parallèle sans bloquer le thread principal.
    fn complete_missing_data(&mut self) -> Task<Message> {
        crate::app::realtime::complete_missing_data(self)
    }
    
    /// Applique les résultats de la complétion des données manquantes
    fn apply_complete_missing_data_results(&mut self, results: Vec<(SeriesId, String, Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_missing_data_results(self, results)
    }

    /// Détecte et complète les gaps dans toutes les séries de manière asynchrone
    fn complete_gaps(&mut self) -> Task<Message> {
        crate::app::realtime::complete_gaps(self)
    }
    
    /// Applique les résultats de la complétion des gaps
    fn apply_complete_gaps_results(&mut self, results: Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_gaps_results(self, results)
    }
    
    /// Met à jour les données en temps réel pour les séries actives
    fn update_realtime(&mut self) -> Task<Message> {
        crate::app::realtime::update_realtime(self)
    }
    
    /// Applique les résultats des mises à jour en temps réel
    fn apply_realtime_updates(&mut self, results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>) {
        crate::app::realtime::apply_realtime_updates(self, results)
    }
    
    /// Sauvegarde l'état complet des panneaux
    fn save_panel_state(&self) {
        use crate::app::panel_persistence::PanelPersistenceState;
        let state = PanelPersistenceState {
            panels: self.panels.clone(),
            active_bottom_section: self.bottom_panel_sections.active_bottom_section,
            active_right_section: self.bottom_panel_sections.active_right_section,
            right_panel_sections: self.bottom_panel_sections.right_panel_sections.clone(),
        };
        if let Err(e) = state.save_to_file("panel_state.json") {
            eprintln!("⚠️ Erreur sauvegarde état panneaux: {}", e);
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        use crate::app::views;
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => views::view_settings(self),
            Some(WindowType::ProviderConfig) => views::view_provider_config(self),
            Some(WindowType::Downloads) => views::view_downloads(self),
            Some(WindowType::Main) | None => views::view_main(self),
        }
    }
}
