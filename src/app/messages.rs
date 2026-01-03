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
    ToggleBollingerBands,
    ToggleMovingAverage,
    // Messages pour modifier les paramètres des indicateurs
    UpdateRSIPeriod(usize),
    UpdateRSIMethod(crate::app::state::RSIMethod),
    UpdateMACDFastPeriod(usize),
    UpdateMACDSlowPeriod(usize),
    UpdateMACDSignalPeriod(usize),
    UpdateBollingerPeriod(usize),
    UpdateBollingerStdDev(f64),
    UpdateMAPeriod(usize),
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
    
    // === Messages de sélection de sections ===
    SelectBottomSection(crate::app::state::BottomPanelSection),
    SelectRightSection(crate::app::state::BottomPanelSection),
    /// Ouvrir le menu contextuel pour une section (avec position du curseur)
    OpenSectionContextMenu(crate::app::state::BottomPanelSection, iced::Point),
    /// Fermer le menu contextuel
    CloseSectionContextMenu,
    /// Déplacer une section vers le panneau de droite
    MoveSectionToRightPanel(crate::app::state::BottomPanelSection),
    /// Déplacer une section vers le panneau du bas
    MoveSectionToBottomPanel(crate::app::state::BottomPanelSection),

    // === Messages de drag des sections ===
    UpdateDragPosition(iced::Point),
    EndDragSection,

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
    
    // === Messages de trading ===
    UpdateOrderQuantity(String),
    UpdateOrderType(crate::app::data::OrderType),
    UpdateLimitPrice(String),
    UpdateTakeProfit(String),
    UpdateStopLoss(String),
    ToggleTPSLEnabled,
    PlaceBuyOrder,
    PlaceSellOrder,
    
    // === Messages des stratégies de trading automatisées ===
    RegisterRSIStrategy,
    RegisterMACrossoverStrategy,
    EnableStrategy(String),
    DisableStrategy(String),
    RemoveStrategy(String),
    // Messages pour l'interface de paramétrage
    ToggleStrategyConfig(String), // Ouvre/ferme le panneau de configuration d'une stratégie
    UpdateStrategyParamInput { strategy_id: String, param_name: String, value: String }, // Valeur temporaire dans l'input
    ToggleStrategyTimeframe { strategy_id: String, timeframe: String }, // Ajoute/retire un timeframe de la sélection
    UpdateStrategyTradingMode { strategy_id: String, trading_mode: crate::app::strategies::strategy::TradingMode }, // Met à jour le mode de trading temporairement
    ApplyStrategyConfig(String), // Applique les modifications d'une stratégie
    CancelStrategyConfig(String), // Annule les modifications d'une stratégie
}

