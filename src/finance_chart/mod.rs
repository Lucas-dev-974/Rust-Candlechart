//! Module de charting financier pour Iced
//! 
//! Architecture modulaire avec séparation stricte :
//! - Core : modèles financiers et calculs
//! - Scale : conversion données → coordonnées écran
//! - Viewport : gestion de la vue visible
//! - Render : logique de dessin
//! - Interaction : gestion des événements utilisateur
//! - State : état de l'application
//! - DataLoader : chargement des données depuis fichiers JSON
//! - AxisCanvas : canvas séparés pour les axes X et Y
//! - ToolsCanvas : état des outils de dessin
//! - ToolsPanelCanvas : panel d'outils à gauche du graphique
//! - Settings : configuration et styles du graphique

pub mod core;
pub mod scale;
pub mod viewport;
pub mod render;
pub mod interaction;
pub mod state;
pub mod widget;
pub mod data_loader;
pub mod axis_canvas;
pub mod tools_canvas;
pub mod tools_panel_canvas;
pub mod series_select;
pub mod settings;
pub mod messages;
pub mod realtime;
pub mod binance_provider;
pub mod provider_config;
pub mod volume_chart;
pub mod volume_axis;
pub mod rsi_chart;
pub mod rsi_axis;
pub mod rsi_data;
pub mod macd_data;
pub mod macd_scaling;
pub mod macd_chart;
pub mod macd_axis;
pub mod indicators;
pub mod axis_style;

pub use state::ChartState;
pub use widget::chart;
pub use data_loader::{load_from_json, load_all_from_directory};
pub use axis_canvas::{x_axis, y_axis, X_AXIS_HEIGHT};
pub use tools_canvas::ToolsState;
pub use tools_panel_canvas::{tools_panel, TOOLS_PANEL_WIDTH};
pub use series_select::series_select_box;
pub use settings::{ChartStyle, SettingsState};
pub use messages::{ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage};
pub use realtime::UpdateResult;
pub use binance_provider::BinanceProvider;
pub use provider_config::{ProviderConfigManager, ProviderType};
pub use volume_chart::volume_chart;
pub use volume_axis::volume_y_axis;
pub use rsi_chart::rsi_chart;
pub use rsi_axis::rsi_y_axis;
pub use macd_chart::macd_chart;
pub use macd_axis::macd_y_axis;
pub use scale::VolumeScale;

