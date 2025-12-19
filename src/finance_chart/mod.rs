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
pub mod simple;

pub use state::ChartState;
pub use widget::chart;
pub use data_loader::{load_from_json, load_all_from_directory};
pub use axis_canvas::{x_axis, y_axis, X_AXIS_HEIGHT, Y_AXIS_WIDTH};
pub use tools_canvas::ToolsState;
pub use tools_panel_canvas::{tools_panel, TOOLS_PANEL_WIDTH};
pub use series_select::series_select_box;
pub use settings::{ChartStyle, SettingsState};
pub use messages::{ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage};

