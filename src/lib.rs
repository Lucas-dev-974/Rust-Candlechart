//! CandleChart - Bibliothèque de visualisation de graphiques financiers
//!
//! Cette bibliothèque fournit des composants pour créer des graphiques en chandeliers (candlesticks)
//! avec le framework Iced.
//!
//! # Exemple d'utilisation
//!
//! ```no_run
//! use candlechart::{
//!     ChartState, chart, load_from_json,
//!     ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
//!     ChartStyle, SettingsState,
//! };
//! use iced::Element;
//!
//! // Charger des données
//! let series = load_from_json("data/BTCUSDT_1h.json").unwrap();
//!
//! // Créer l'état du graphique
//! let mut chart_state = ChartState::new(1200.0, 800.0);
//! chart_state.add_series(series);
//!
//! // Créer les widgets
//! let tools_state = ToolsState::default();
//! let settings_state = SettingsState::default();
//! let chart_style = ChartStyle::default();
//!
//! // Créer le widget graphique
//! let chart_widget: Element<_> = chart(
//!     &chart_state,
//!     &tools_state,
//!     &settings_state,
//!     &chart_style,
//! );
//! ```
//!
//! # Modules principaux
//!
//! - `finance_chart::core` : Structures de données financières (Candle, TimeSeries, SeriesManager)
//! - `finance_chart::scale` : Conversion prix/temps → coordonnées écran
//! - `finance_chart::viewport` : Gestion de la vue visible (zoom, pan)
//! - `finance_chart::render` : Fonctions de rendu des éléments graphiques
//! - `finance_chart::state` : État global du graphique
//! - `finance_chart::widget` : Widget canvas principal
//! - `finance_chart::data_loader` : Chargement de données depuis JSON
//! - `finance_chart::settings` : Configuration et styles

pub mod finance_chart;

// Ré-exporter les éléments publics principaux pour faciliter l'utilisation
pub use finance_chart::{
    // État
    ChartState,
    
    // Widgets
    chart,
    x_axis, y_axis, X_AXIS_HEIGHT, Y_AXIS_WIDTH,
    tools_panel, TOOLS_PANEL_WIDTH,
    series_select_box,
    
    // États
    ToolsState,
    SettingsState,
    
    // Styles
    ChartStyle,
    
    // Chargement de données
    load_from_json, load_all_from_directory,
    
    // Messages
    ChartMessage, YAxisMessage, XAxisMessage,
    ToolsPanelMessage, SeriesPanelMessage,
    
    // Temps réel
    UpdateResult, BinanceProvider,
};

// Ré-exporter les types core depuis les sous-modules
pub use finance_chart::core::{
    Candle, TimeSeries, SeriesId, SeriesData, SeriesManager,
};

// Ré-exporter les types scale
pub use finance_chart::scale::{
    PriceScale, TimeScale,
};

// Ré-exporter Viewport
pub use finance_chart::viewport::Viewport;

// Ré-exporter le trait RealtimeDataProvider (utilisé comme contrainte de type générique)
pub use finance_chart::realtime::RealtimeDataProvider;

