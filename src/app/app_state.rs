//! État principal de l'application
//!
//! Ce module définit la structure ChartApp et ses méthodes de base (new, title, theme, subscription).

use iced::{Task, Theme, window, Subscription, Size};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use crate::finance_chart::{
    ChartState, ToolsState, SettingsState, ChartStyle,
    BinanceProvider, ProviderConfigManager, ProviderType,
    core::SeriesId,
};
use crate::app::{
    constants::*,
    window_manager::{WindowManager, WindowType},
    messages::Message,
    data_loading,
    panel_state::{PanelsState, MIN_PANEL_SIZE},
    bottom_panel_sections::BottomPanelSectionsState,
    account_type::AccountTypeState,
    account_info::AccountInfo,
    panel_persistence::PanelPersistenceState,
    download_manager::DownloadManager,
};

/// Application principale - possède directement tout l'état (pas de Rc<RefCell>)
pub struct ChartApp {
    // État possédé directement
    pub chart_state: ChartState,
    pub tools_state: ToolsState,
    pub settings_state: SettingsState,
    pub chart_style: ChartStyle,
    
    // Gestion des fenêtres
    pub windows: WindowManager,
    
    // État temporaire pour l'édition des settings
    pub editing_style: Option<ChartStyle>,
    pub editing_color_index: Option<usize>,
    
    // Mode temps réel - Arc pour partage efficace sans clonage coûteux
    pub binance_provider: Arc<BinanceProvider>,
    pub realtime_enabled: bool,
    
    // Configuration des providers
    pub provider_config: ProviderConfigManager,
    
    // État temporaire pour la fenêtre de configuration des providers
    pub editing_provider_token: HashMap<ProviderType, String>,
    
    // Compteur de version pour forcer le re-render du canvas
    pub render_version: u64,
    
    // État des panneaux latéraux
    pub panels: PanelsState,
    
    // État des sections du panneau du bas
    pub bottom_panel_sections: BottomPanelSectionsState,
    
    // État du drag & drop des sections
    pub dragging_section: Option<crate::app::bottom_panel_sections::BottomPanelSection>,
    pub drag_from_right_panel: bool, // Indique si le drag a commencé depuis le panneau de droite
    pub drag_over_right_panel: bool, // Indique si on survole le panneau de droite pendant le drag
    pub drag_position: Option<iced::Point>, // Position de la souris pendant le drag
    
    // État du type de compte
    pub account_type: AccountTypeState,
    
    // Informations du compte de trading
    pub account_info: AccountInfo,
    
    // État de connexion au provider
    pub provider_connection_status: Option<bool>, // None = non testé, Some(true) = connecté, Some(false) = non connecté
    pub provider_connection_testing: bool, // Indique si un test de connexion est en cours
    
    // État de l'onglet d'indicateurs
    pub indicators_panel_open: bool, // Indique si l'onglet d'indicateurs est ouvert
    
    // État des téléchargements
    pub download_manager: DownloadManager, // Gestionnaire de téléchargements multiples
}

/// État de progression d'un téléchargement
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub series_id: SeriesId,
    pub current_count: usize,
    pub estimated_total: usize,
    pub current_start: i64,
    pub target_end: i64,
    pub gaps_remaining: Vec<(i64, i64)>,
    pub paused: bool, // Indique si le téléchargement est en pause
}

impl ChartApp {
    pub fn new() -> (Self, Task<Message>) {
        // Créer l'état initial vide - les données seront chargées de manière asynchrone
        let chart_state = ChartState::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT);
        
        // Créer l'état des outils (dessins chargés de manière synchrone car rapide)
        let mut tools_state = ToolsState::default();
        match tools_state.load_from_file("drawings.json") {
            Ok(()) => {
                println!(
                    "✅ Dessins chargés: {} rectangles, {} lignes horizontales",
                    tools_state.rectangles.len(),
                    tools_state.horizontal_lines.len()
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Ignorer seulement les erreurs "fichier non trouvé"
                if !error_msg.contains("No such file") 
                    && !error_msg.contains("cannot find")
                    && !error_msg.contains("not found") {
                    eprintln!("⚠️ Impossible de charger les dessins: {}", e);
                }
            }
        }

        // Charger le style (rapide, synchrone)
        let chart_style = match ChartStyle::load_from_file("chart_style.json") {
            Ok(style) => {
                println!("✅ Style chargé depuis chart_style.json");
                style
            }
            Err(_) => ChartStyle::default(),
        };

        // Charger la configuration des providers (rapide, synchrone)
        let provider_config = match ProviderConfigManager::load_from_file("provider_config.json") {
            Ok(config) => {
                println!("✅ Configuration des providers chargée depuis provider_config.json");
                config
            }
            Err(_) => {
                println!("ℹ️ Configuration des providers par défaut utilisée");
                ProviderConfigManager::new()
            }
        };

        // Créer le provider Binance avec le token configuré (Arc pour partage efficace)
        let binance_provider = Arc::new(if let Some(config) = provider_config.active_config() {
            BinanceProvider::with_token(config.api_token.clone())
        } else {
            BinanceProvider::new()
        });

        // Ouvrir la fenêtre principale IMMÉDIATEMENT
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT),
            ..Default::default()
        });

        // Créer une Task pour charger les séries de manière asynchrone
        let load_series_task = data_loading::create_load_series_task(binance_provider.clone());

        (
            Self { 
                chart_state, 
                tools_state, 
                settings_state: SettingsState::default(),
                chart_style,
                provider_config,
                editing_provider_token: HashMap::new(),
                windows: WindowManager::new(main_id),
                editing_style: None,
                editing_color_index: None,
                binance_provider,
                realtime_enabled: true, // Activer le mode temps réel par défaut
                render_version: 0,
                panels: Self::load_panels_state(),
                bottom_panel_sections: Self::load_bottom_panel_sections(),
                dragging_section: None,
                drag_from_right_panel: false,
                drag_over_right_panel: false,
                drag_position: None,
                account_type: AccountTypeState::new(),
                account_info: AccountInfo::new(),
                provider_connection_status: None,
                provider_connection_testing: false,
                indicators_panel_open: false,
                download_manager: DownloadManager::new(),
            },
            Task::batch(vec![
                open_task.map(Message::MainWindowOpened),
                load_series_task,
            ]),
        )
    }
    
    /// Charge l'état des panneaux depuis le fichier
    fn load_panels_state() -> PanelsState {
        match PanelPersistenceState::load_from_file("panel_state.json") {
            Ok(state) => {
                println!("✅ État des panneaux chargé depuis panel_state.json");
                // Restaurer les valeurs par défaut pour les champs non sérialisés
                let mut panels = state.panels;
                // Lire le JSON pour vérifier si macd existe
                let panels_state_json = std::fs::read_to_string("panel_state.json").unwrap_or_default();
                // Restaurer les valeurs pour le panneau de droite
                panels.right.min_size = MIN_PANEL_SIZE;
                panels.right.max_size = 500.0;
                panels.right.is_resizing = false;
                panels.right.resize_start = None;
                panels.right.focused = false;
                // Restaurer les valeurs pour le panneau du bas
                panels.bottom.min_size = MIN_PANEL_SIZE;
                panels.bottom.max_size = 400.0;
                panels.bottom.is_resizing = false;
                panels.bottom.resize_start = None;
                panels.bottom.focused = false;
                // Restaurer les valeurs pour le panneau de volume
                panels.volume.min_size = MIN_PANEL_SIZE;
                panels.volume.max_size = 400.0;
                panels.volume.is_resizing = false;
                panels.volume.resize_start = None;
                panels.volume.focused = false;
                // Restaurer les valeurs pour le panneau RSI
                panels.rsi.min_size = MIN_PANEL_SIZE;
                panels.rsi.max_size = 400.0;
                panels.rsi.is_resizing = false;
                panels.rsi.resize_start = None;
                panels.rsi.focused = false;
                
                // Initialiser le panneau MACD avec des valeurs par défaut
                panels.macd.min_size = MIN_PANEL_SIZE;
                panels.macd.max_size = 400.0;
                panels.macd.is_resizing = false;
                panels.macd.resize_start = None;
                panels.macd.focused = false;
                // Le MACD panel est masqué par défaut si non présent dans le JSON
                if !panels_state_json.contains("\"macd\"") {
                    panels.macd.visible = false;
                }
                
                panels
            }
            Err(_) => {
                PanelsState::new()
            }
        }
    }
    
    /// Charge l'état des sections du panneau du bas depuis le fichier
    fn load_bottom_panel_sections() -> BottomPanelSectionsState {
        match PanelPersistenceState::load_from_file("panel_state.json") {
            Ok(state) => {
                println!("✅ Section active du panneau chargée depuis panel_state.json");
                BottomPanelSectionsState {
                    active_bottom_section: state.active_bottom_section,
                    active_right_section: state.active_right_section,
                    right_panel_sections: state.right_panel_sections,
                }
            }
            Err(_) => {
                BottomPanelSectionsState::new()
            }
        }
    }

    pub fn title(&self, window_id: window::Id) -> String {
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => String::from("Settings - Style Chart"),
            Some(WindowType::ProviderConfig) => String::from("Provider Configuration"),
            Some(WindowType::Downloads) => {
                let count = self.download_manager.count();
                if count > 0 {
                    format!("Téléchargements ({})", count)
                } else {
                    String::from("Téléchargements")
                }
            }
            Some(WindowType::Main) | None => {
                // Afficher le symbole de la série active, ou un titre par défaut
                if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                    active_series.symbol.clone()
                } else {
                    String::from("CandleChart")
                }
            }
        }
    }

    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.realtime_enabled {
            // Subscription pour les mises à jour en temps réel
            Subscription::batch(vec![
                iced::time::every(Duration::from_secs_f64(REALTIME_UPDATE_INTERVAL_SECS))
                    .map(|_| Message::RealtimeUpdate),
                window::close_events().map(Message::WindowClosed),
            ])
        } else {
            window::close_events().map(Message::WindowClosed)
        }
    }
}

