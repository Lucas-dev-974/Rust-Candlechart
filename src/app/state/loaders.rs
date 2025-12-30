//! Chargeurs d'état
//!
//! Ce module contient toute la logique de chargement des états persistants
//! depuis les fichiers JSON.

use crate::app::{
    state::{
        PanelsState, MIN_PANEL_SIZE, BottomPanelSectionsState, TradingState,
        BottomPanelSection,
    },
    persistence::{PanelPersistenceState, TimeframePersistenceState},
    data::TradeHistory,
};

/// Charge l'état des panneaux depuis le fichier
pub fn load_panels_state() -> PanelsState {
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

/// Charge l'état de trading depuis le fichier
pub fn load_trading_state() -> TradingState {
    match TradeHistory::load_from_file("paper_trading.json") {
        Ok(trade_history) => {
            println!("✅ Historique de trading chargé depuis paper_trading.json ({} trades, {} positions)", 
                trade_history.trades.len(), trade_history.open_positions.len());
            TradingState {
                order_quantity: String::from("0.001"),
                order_type: crate::app::data::OrderType::Market,
                limit_price: String::new(),
                take_profit: String::new(),
                stop_loss: String::new(),
                tp_sl_enabled: true,
                trade_history,
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Ignorer seulement les erreurs "fichier non trouvé"
            if !error_msg.contains("No such file") 
                && !error_msg.contains("cannot find")
                && !error_msg.contains("not found") {
                eprintln!("⚠️ Impossible de charger l'historique de trading: {}", e);
            }
            TradingState::new()
        }
    }
}

/// Charge l'état des sections du panneau du bas depuis le fichier
pub fn load_bottom_panel_sections() -> BottomPanelSectionsState {
    match PanelPersistenceState::load_from_file("panel_state.json") {
        Ok(state) => {
            println!("✅ Section active du panneau chargée depuis panel_state.json");
            let mut sections_state = BottomPanelSectionsState {
                active_bottom_section: state.active_bottom_section,
                active_right_section: state.active_right_section,
                right_panel_sections: state.right_panel_sections,
            };
            
            // S'assurer que Orders et TradeHistory sont dans le panneau de droite
            if !sections_state.right_panel_sections.contains(&BottomPanelSection::Orders) {
                sections_state.right_panel_sections.push(BottomPanelSection::Orders);
            }
            if !sections_state.right_panel_sections.contains(&BottomPanelSection::TradeHistory) {
                sections_state.right_panel_sections.push(BottomPanelSection::TradeHistory);
            }
            // Si aucune section n'est active dans le panneau de droite, activer Orders
            if sections_state.active_right_section.is_none() {
                sections_state.active_right_section = Some(BottomPanelSection::Orders);
            }
            
            sections_state
        }
        Err(_) => {
            BottomPanelSectionsState::new()
        }
    }
}

/// Charge les dessins depuis le fichier
pub fn load_tools_state() -> crate::finance_chart::ToolsState {
    let mut tools_state = crate::finance_chart::ToolsState::default();
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
    tools_state
}

/// Charge le style du graphique depuis le fichier
pub fn load_chart_style() -> crate::finance_chart::ChartStyle {
    match crate::finance_chart::ChartStyle::load_from_file("chart_style.json") {
        Ok(style) => {
            println!("✅ Style chargé depuis chart_style.json");
            style
        }
        Err(_) => crate::finance_chart::ChartStyle::default(),
    }
}

/// Charge la configuration des providers depuis le fichier
pub fn load_provider_config() -> crate::finance_chart::ProviderConfigManager {
    match crate::finance_chart::ProviderConfigManager::load_from_file("provider_config.json") {
        Ok(config) => {
            println!("✅ Configuration des providers chargée depuis provider_config.json");
            config
        }
        Err(_) => {
            println!("ℹ️ Configuration des providers par défaut utilisée");
            crate::finance_chart::ProviderConfigManager::new()
        }
    }
}

/// Charge le timeframe sélectionné depuis le fichier
pub fn load_timeframe() -> Option<String> {
    match TimeframePersistenceState::load_from_file("timeframe.json") {
        Ok(state) => {
            println!("✅ Timeframe chargé depuis timeframe.json: {}", state.interval);
            Some(state.interval)
        }
        Err(_) => {
            None
        }
    }
}



