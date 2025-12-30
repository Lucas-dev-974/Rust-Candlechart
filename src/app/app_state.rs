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
    utils::constants::{MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, REALTIME_UPDATE_INTERVAL_SECS},
    window_manager::{WindowManager, WindowType},
    messages::Message,
    data::data_loading,
    state::{
        AccountTypeState, AccountInfo, TradingState,
        UiState, IndicatorState,
        loaders::{
            load_panels_state, load_trading_state, load_bottom_panel_sections,
            load_tools_state, load_chart_style, load_provider_config,
        },
    },
    data::DownloadManager,
    strategies::manager::StrategyManager,
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
    
    // État de l'interface utilisateur (panneaux, sections, drag, menus)
    pub ui: UiState,
    
    // État du type de compte
    pub account_type: AccountTypeState,
    
    // Informations du compte de trading
    pub account_info: AccountInfo,
    
    // État de connexion au provider
    pub provider_connection_status: Option<bool>, // None = non testé, Some(true) = connecté, Some(false) = non connecté
    pub provider_connection_testing: bool, // Indique si un test de connexion est en cours
    
    // État des indicateurs techniques
    pub indicators: IndicatorState,
    
    // État des téléchargements
    pub download_manager: DownloadManager, // Gestionnaire de téléchargements multiples
    
    // État de trading
    pub trading_state: TradingState,
    
    // Gestionnaire de stratégies de trading automatisées
    pub strategy_manager: StrategyManager,
    
    // État temporaire pour l'édition des stratégies
    pub editing_strategies: HashMap<String, StrategyEditingState>,
}

/// État temporaire pour l'édition d'une stratégie
#[derive(Debug, Clone)]
pub struct StrategyEditingState {
    /// Indique si le panneau de configuration est ouvert
    pub expanded: bool,
    /// Valeurs temporaires des paramètres (nom -> valeur en string)
    pub param_values: HashMap<String, String>,
    /// Timeframes sélectionnés temporairement
    pub selected_timeframes: Vec<String>,
    /// Mode de trading sélectionné temporairement
    pub trading_mode: crate::app::strategies::strategy::TradingMode,
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
        
        // Charger les états depuis les fichiers
        let tools_state = load_tools_state();
        let chart_style = load_chart_style();
        let provider_config = load_provider_config();

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
                ui: UiState {
                    panels: load_panels_state(),
                    bottom_panel_sections: load_bottom_panel_sections(),
                    section_context_menu: None,
                    drag_overlay: None,
                    indicators_panel_open: false,
                },
                account_type: AccountTypeState::new(),
                account_info: AccountInfo::new(),
                provider_connection_status: None,
                provider_connection_testing: false,
                indicators: IndicatorState::new(),
                download_manager: DownloadManager::new(),
                trading_state: load_trading_state(),
                strategy_manager: StrategyManager::new_or_load("strategies.json"),
                editing_strategies: HashMap::new(),
            },
            Task::batch(vec![
                open_task.map(Message::MainWindowOpened),
                load_series_task,
            ]),
        )
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

