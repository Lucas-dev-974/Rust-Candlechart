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
    OpenDownloads,
    DownloadsWindowOpened(window::Id),
    
    // === Messages des settings ===
    SelectColor(usize, SerializableColor),
    ApplySettings,
    CancelSettings,
    ToggleColorPicker(usize),
    ToggleAutoScroll,
    
    // === Messages temps réel ===
    RealtimeUpdate,
    RealtimeUpdateComplete(Vec<(SeriesId, String, Result<Option<Candle>, String>)>),
    #[allow(dead_code)] // Utilisé dans main.rs mais jamais construit directement
    CompleteMissingData,
    CompleteMissingDataComplete(Vec<(SeriesId, String, Result<Vec<Candle>, String>)>),
    #[allow(dead_code)] // Utilisé dans main.rs mais jamais construit directement
    LoadFullHistory(SeriesId),
    LoadFullHistoryComplete(SeriesId, String, Result<Vec<Candle>, String>),
    /// Démarrer le téléchargement par batch avec la liste des gaps
    StartBatchDownload(SeriesId, Vec<(i64, i64)>, usize), // series_id, gaps, estimated_total
    /// Résultat d'un batch de téléchargement (avec next_start pour continuer)
    BatchDownloadResult(SeriesId, Vec<Candle>, usize, usize, i64), // series_id, candles, count, estimated, next_start
    /// Téléchargement terminé
    DownloadComplete(SeriesId),
    /// Mettre en pause un téléchargement
    PauseDownload(SeriesId),
    /// Reprendre un téléchargement
    ResumeDownload(SeriesId),
    /// Arrêter un téléchargement
    StopDownload(SeriesId),
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
    
    // === Messages des panneaux latéraux ===
    ToggleVolumePanel,
    ToggleRSIPanel,
    ToggleMACDPanel,
    StartResizeRightPanel(f32),
    StartResizeBottomPanel(f32),
    StartResizeVolumePanel(f32),
    StartResizeRSIPanel(f32),
    StartResizeMACDPanel(f32),
    UpdateResizeRightPanel(f32),
    UpdateResizeBottomPanel(f32),
    UpdateResizeVolumePanel(f32),
    UpdateResizeRSIPanel(f32),
    UpdateResizeMACDPanel(f32),
    EndResizeRightPanel,
    EndResizeBottomPanel,
    EndResizeVolumePanel,
    EndResizeRSIPanel,
    EndResizeMACDPanel,
    
    
    // === Messages de drag & drop ===
    StartDragSection(crate::app::bottom_panel_sections::BottomPanelSection),
    UpdateDragPosition(iced::Point),
    EndDragSection,
    DragEnterRightPanel,
    DragExitRightPanel,
    
    // === Messages du type de compte ===
    ToggleAccountType,
    
    // === Messages de test de connexion au provider ===
    TestProviderConnection,
    ProviderConnectionTestComplete(Result<(), String>),
    
    // === Messages de focus des panneaux ===
    SetRightPanelFocus(bool),
    SetBottomPanelFocus(bool),
    SetVolumePanelFocus(bool),
    SetRSIPanelFocus(bool),
    SetMACDPanelFocus(bool),
    ClearPanelFocus,
}

