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
};
use crate::app::{
    constants::*,
    window_manager::{WindowManager, WindowType},
    messages::Message,
    data_loading,
    panel_state::PanelsState,
    bottom_panel_sections::BottomPanelSectionsState,
    account_type::AccountTypeState,
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
    
    // État du type de compte
    pub account_type: AccountTypeState,
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
        let load_series_task = data_loading::create_load_series_task();

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
                panels: PanelsState::new(),
                bottom_panel_sections: BottomPanelSectionsState::new(),
                account_type: AccountTypeState::new(),
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

