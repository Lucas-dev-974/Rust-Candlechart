//! Messages de l'application Iced
//!
//! Ce module définit tous les messages que l'application peut recevoir et traiter.

use iced::window;
use crate::finance_chart::{
    ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage,
    settings::SerializableColor,
    core::{SeriesId, Candle, SeriesData},
    ProviderType,
};

/// Messages de l'application
#[derive(Debug, Clone)]
pub enum Message {
    // === Messages du graphique ===
    Chart(ChartMessage),
    
    // === Messages des axes ===
    YAxis(YAxisMessage),
    XAxis(XAxisMessage),
    
    // === Messages du panel d'outils ===
    ToolsPanel(ToolsPanelMessage),
    
    // === Messages du panel de séries ===
    SeriesPanel(SeriesPanelMessage),
    
    // === Messages de fenêtres ===
    OpenSettings,
    SettingsWindowOpened(window::Id),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
    
    // === Messages des settings ===
    SelectColor(usize, SerializableColor),
    ApplySettings,
    CancelSettings,
    ToggleColorPicker(usize),
    ToggleAutoScroll,
    
    // === Messages temps réel ===
    RealtimeUpdate,
    RealtimeUpdateComplete(Vec<(SeriesId, String, Result<Option<Candle>, String>)>),
    CompleteMissingData,
    CompleteMissingDataComplete(Vec<(SeriesId, String, Result<Vec<Candle>, String>)>),
    #[allow(dead_code)] // Utilisé dans le match de main.rs (ligne 308)
    CompleteGaps,
    CompleteGapsComplete(Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>),
    SaveSeriesComplete(Vec<(String, Result<(), String>)>),
    #[allow(dead_code)] // Utilisé dans le match de main.rs (ligne 77)
    LoadSeriesFromDirectory,
    LoadSeriesFromDirectoryComplete(Result<Vec<SeriesData>, String>),
    
    // === Messages de configuration des providers ===
    OpenProviderConfig,
    ProviderConfigWindowOpened(window::Id),
    SelectProvider(ProviderType),
    UpdateProviderToken(ProviderType, String),
    ApplyProviderConfig,
    CancelProviderConfig,
}

