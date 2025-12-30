//! Module de charting financier pour Iced
//! 
//! Architecture modulaire avec séparation stricte :
//! - Core : modèles financiers et calculs
//! - Scale : conversion données → coordonnées écran
//! - Viewport : gestion de la vue visible
//! - Render : logique de dessin
//! - Interaction : gestion des événements utilisateur
//! - State : état de l'application
//! - Axis : canvas et styles pour les axes X/Y
//! - Tools : outils de dessin et panel
//! - Providers : providers de données (Binance, etc.)
//! - Realtime : mise à jour en temps réel
//! - Indicators : indicateurs techniques (RSI, MACD, Volume, EMA)

pub mod core;
pub mod scale;
pub mod viewport;
pub mod render;
pub mod interaction;
pub mod state;
pub mod widget;
pub mod data_loader;
pub mod series_select;
pub mod settings;
pub mod messages;

// Modules réorganisés en dossiers
pub mod axis;
pub mod tools;
pub mod providers;
pub mod realtime;
pub mod indicators;

// Ré-exports principaux
pub use state::ChartState;
pub use widget::{chart, chart_with_trades, chart_with_trading, chart_with_trades_and_trading};
pub use data_loader::{load_from_json, load_all_from_directory, is_directory_empty, save_to_json};
pub use series_select::series_select_box;
pub use settings::{ChartStyle, SettingsState};
pub use messages::{ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage};

// Ré-exports depuis axis/
pub use axis::{x_axis, y_axis, X_AXIS_HEIGHT};

// Ré-exports depuis tools/
pub use tools::{tools_panel, ToolsState, TOOLS_PANEL_WIDTH};

// Ré-exports depuis providers/
pub use providers::{BinanceProvider, ProviderConfigManager, ProviderType};

// Ré-exports depuis realtime/
pub use realtime::UpdateResult;

// Ré-exports depuis indicators/
pub use indicators::rsi::rsi_chart;
pub use indicators::rsi::rsi_y_axis;
pub use indicators::macd::macd_chart;
pub use indicators::macd::macd_y_axis;
pub use indicators::volume::volume_chart;
pub use indicators::volume::volume_y_axis;
