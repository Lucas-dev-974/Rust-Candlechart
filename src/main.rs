mod finance_chart;
mod app;

use iced::{Task, Size, window, exit, Element};
use std::time::Duration;
use std::sync::Arc;
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
            
            // === Gestion des messages du panel de séries ===
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                // Trouver le SeriesId correspondant au nom
                let series_id_opt = self.chart_state.series_manager.all_series()
                    .find(|s| s.full_name() == series_name)
                    .map(|s| s.id.clone());
                
                if let Some(series_id) = series_id_opt {
                    // Activer uniquement cette série (désactive toutes les autres)
                    self.chart_state.series_manager.activate_only_series(series_id);
                    // Mettre à jour le viewport après activation
                    self.chart_state.update_viewport_from_series();
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
                        if self.chart_state.series_manager.total_count() == 0 {
                            eprintln!("⚠️ Aucune série chargée. Vérifiez que le dossier 'data' contient des fichiers JSON.");
                            return Task::none();
                        }
                        
                        // Après avoir chargé les séries, compléter les données manquantes
                        println!("🔄 Démarrage de la complétion des données manquantes...");
                        return Task::perform(
                            async {
                                // Attendre un peu pour que l'UI soit prête
                                tokio::time::sleep(Duration::from_millis(300)).await;
                                Message::CompleteMissingData
                            },
                            |_| Message::CompleteMissingData,
                        );
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
            Message::ToggleRightPanel => {
                self.panels.right.toggle_visibility();
                Task::none()
            }
            Message::ToggleBottomPanel => {
                self.panels.bottom.toggle_visibility();
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
            Message::SelectBottomPanelSection(section) => {
                self.bottom_panel_sections.set_active_section(section);
                // Sauvegarder l'état complet des panneaux
                self.save_panel_state();
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
            Message::ClearPanelFocus => {
                self.panels.right.set_focused(false);
                self.panels.bottom.set_focused(false);
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
            Message::SetAccountType(account_type) => {
                self.account_type.set_account_type(account_type);
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
            active_section: self.bottom_panel_sections.active_section,
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
            Some(WindowType::Main) | None => views::view_main(self),
        }
    }
}
