//! État de l'application
//!
//! Ce module contient tous les types et structures d'état de l'application.

mod panel_state;
mod account_info;
mod account_type;
mod bottom_panel_sections;
mod trading_state;
mod indicator_params;
pub mod loaders;
mod ui_state;
mod indicator_state;

pub use panel_state::{PanelsState, MIN_PANEL_SIZE};
pub use account_info::AccountInfo;
pub use account_type::{AccountType, AccountTypeState};
pub use bottom_panel_sections::{BottomPanelSection, BottomPanelSectionsState};
pub use trading_state::TradingState;
pub use indicator_params::IndicatorParams;
pub use ui_state::UiState;
pub use indicator_state::IndicatorState;
pub use loaders::{
    load_panels_state, load_trading_state, load_bottom_panel_sections,
    load_tools_state, load_chart_style, load_provider_config,
};



