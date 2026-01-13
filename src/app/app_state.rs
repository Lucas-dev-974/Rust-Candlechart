//! État principal de l'application
//!
//! Ce module définit la structure ChartApp et ses méthodes principales (new, update, view, title, theme, subscription).

use iced::{Task, Theme, window, Subscription, Size, Element};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use crate::finance_chart::{
    ChartState, ToolsState, SettingsState, ChartStyle,
    BinanceProvider, ProviderConfigManager, ProviderType,
    core::{SeriesId, Candle},
    SeriesPanelMessage,
};
use crate::app::{
    utils::constants::{MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, REALTIME_UPDATE_INTERVAL_SECS},
    window_manager::{WindowManager, WindowType},
    messages::Message,
    data::data_loading,
    state::{
        AccountTypeState, AccountInfo, TradingState,
        UiState, IndicatorState,
        loaders::{
            load_panels_state, load_trading_state, load_bottom_panel_sections,
            load_tools_state, load_chart_style, load_provider_config,
        },
    },
    data::DownloadManager,
    strategies::manager::StrategyManager,
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
    pub editing_provider_secret: HashMap<ProviderType, String>,
    
    // Compteur de version pour forcer le re-render du canvas
    pub render_version: u64,
    
    // État de l'interface utilisateur (panneaux, sections, menus)
    pub ui: UiState,
    
    // État du type de compte
    pub account_type: AccountTypeState,
    
    // Informations du compte de trading
    pub account_info: AccountInfo,
    
    // État de connexion au provider
    pub provider_connection_status: Option<bool>, // None = non testé, Some(true) = connecté, Some(false) = non connecté
    pub provider_connection_testing: bool, // Indique si un test de connexion est en cours
    
    // État des indicateurs techniques
    pub indicators: IndicatorState,
    
    // État des téléchargements
    pub download_manager: DownloadManager, // Gestionnaire de téléchargements multiples
    
    // État de trading
    pub trading_state: TradingState,
    
    // Gestionnaire de stratégies de trading automatisées
    pub strategy_manager: StrategyManager,
    
    // État temporaire pour l'édition des stratégies
    pub editing_strategies: HashMap<String, StrategyEditingState>,
    
    // État de la fenêtre Actifs
    pub assets: Vec<crate::finance_chart::providers::binance::BinanceSymbol>,
    pub assets_loading: bool,
    pub selected_assets: std::collections::HashSet<String>, // Symboles des actifs sélectionnés
    pub selected_asset_symbol: Option<String>, // Dernier symbole sélectionné depuis le pick_list (pour l'affichage)
}

/// État temporaire pour l'édition d'une stratégie
#[derive(Debug, Clone)]
pub struct StrategyEditingState {
    /// Indique si le panneau de configuration est ouvert
    pub expanded: bool,
    /// Valeurs temporaires des paramètres (nom -> valeur en string)
    pub param_values: HashMap<String, String>,
    /// Timeframes sélectionnés temporairement
    pub selected_timeframes: Vec<String>,
    /// Mode de trading sélectionné temporairement
    pub trading_mode: crate::app::strategies::strategy::TradingMode,
}

/// État de progression d'un téléchargement
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub series_id: SeriesId,
    pub current_count: usize,
    pub estimated_total: usize,
    pub current_start: i64,
    pub target_end: i64,
    pub gaps_remaining: Vec<(i64, i64)>,
    pub paused: bool, // Indique si le téléchargement est en pause
}

impl ChartApp {
    pub fn new() -> (Self, Task<Message>) {
        // Créer l'état initial vide - les données seront chargées de manière asynchrone
        let chart_state = ChartState::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT);
        
        // Charger les états depuis les fichiers
        let tools_state = load_tools_state();
        let chart_style = load_chart_style();
        let provider_config = load_provider_config();

        // Créer le provider Binance avec le token et la clé secrète configurés (Arc pour partage efficace)
        let binance_provider = Arc::new(if let Some(config) = provider_config.active_config() {
            BinanceProvider::with_token_and_secret(
                config.api_token.clone(),
                config.api_secret.clone()
            )
        } else {
            BinanceProvider::new()
        });

        // Ouvrir la fenêtre principale IMMÉDIATEMENT
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT),
            ..Default::default()
        });

        // Créer une Task pour charger les séries de manière asynchrone
        let load_series_task = data_loading::create_load_series_task(binance_provider.clone());

        (
            Self { 
                chart_state, 
                tools_state, 
                settings_state: SettingsState::default(),
                chart_style,
                provider_config,
                editing_provider_token: HashMap::new(),
                editing_provider_secret: HashMap::new(),
                windows: WindowManager::new(main_id),
                editing_style: None,
                editing_color_index: None,
                binance_provider,
                realtime_enabled: true, // Activer le mode temps réel par défaut
                render_version: 0,
                ui: UiState {
                    panels: load_panels_state(),
                    bottom_panel_sections: load_bottom_panel_sections(),
                    section_context_menu: None,
                    chart_context_menu: None,
                    indicators_panel_open: false,
                    backtest_state: crate::app::state::backtest::BacktestState::new(),
                },
                account_type: AccountTypeState::new(),
                account_info: AccountInfo::new(),
                provider_connection_status: None,
                provider_connection_testing: false,
                indicators: IndicatorState::new(),
                download_manager: DownloadManager::new(),
                trading_state: load_trading_state(),
                strategy_manager: StrategyManager::new_or_load("strategies.json"),
                editing_strategies: HashMap::new(),
                assets: Vec::new(),
                assets_loading: false,
                selected_assets: {
                    // Charger les actifs sélectionnés depuis le fichier
                    crate::app::persistence::SelectedAssetsPersistenceState::load_from_file("selected_assets.json")
                        .map(|state| state.to_hashset())
                        .unwrap_or_else(|_| std::collections::HashSet::new())
                },
                selected_asset_symbol: {
                    // Restaurer le symbole sauvegardé depuis timeframe.json
                    crate::app::persistence::TimeframePersistenceState::load_from_file("timeframe.json")
                        .ok()
                        .and_then(|state| state.symbol)
                },
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
            Some(WindowType::Downloads) => {
                let count = self.download_manager.count();
                if count > 0 {
                    format!("Téléchargements ({})", count)
                } else {
                    String::from("Téléchargements")
                }
            }
            Some(WindowType::Assets) => {
                let count = self.assets.len();
                if count > 0 {
                    format!("Actifs disponibles ({})", count)
                } else {
                    String::from("Actifs disponibles")
                }
            }
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
        let mut subscriptions = vec![window::close_events().map(Message::WindowClosed)];
        
        if self.realtime_enabled {
            // Subscription pour les mises à jour en temps réel
            subscriptions.push(
                iced::time::every(Duration::from_secs_f64(REALTIME_UPDATE_INTERVAL_SECS))
                    .map(|_| Message::RealtimeUpdate)
            );
        }
        
        // Subscription pour les ticks du backtest si en cours de lecture
        if self.ui.backtest_state.is_playing {
            let speed_ms = self.ui.backtest_state.playback_speed_ms;
            subscriptions.push(
                iced::time::every(Duration::from_millis(speed_ms))
                    .map(|_| Message::BacktestTick)
            );
        }
        
        Subscription::batch(subscriptions)
    }

    /// Traite les messages de l'application
    pub fn update(&mut self, message: Message) -> Task<Message> {
        use crate::app::handlers::*;
        
        match message {
            // === Gestion des messages du graphique ===
            Message::Chart(chart_msg) => {
                handle_chart_message(self, chart_msg);
                // Si un clic droit a été détecté, ouvrir le menu contextuel
                if let Some(position) = self.ui.chart_context_menu.take() {
                    return handle_open_chart_context_menu(self, position);
                }
                Task::none()
            }
            Message::ResetView => handle_reset_view(self),
            Message::OpenChartContextMenu(position) => handle_open_chart_context_menu(self, position),
            Message::CloseChartContextMenu => handle_close_chart_context_menu(self),
            
            // === Gestion des messages des axes ===
            Message::YAxis(msg) => handle_yaxis_message(self, msg),
            Message::XAxis(msg) => handle_xaxis_message(self, msg),
            
            // === Gestion des messages du panel d'outils ===
            Message::ToolsPanel(msg) => handle_tools_panel_message(self, msg),
            
            // === Gestion des messages du panel de séries ===
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                handle_select_series_by_name(self, series_name)
            }
            
            // === Gestion des fenêtres ===
            Message::MainWindowOpened(_id) => Task::none(),
            
            Message::LoadSeriesFromDirectory => Task::none(),
            
            Message::LoadSeriesFromDirectoryComplete(result) => {
                handle_load_series_complete(self, result)
            }
            
            Message::OpenSettings => handle_open_settings(self),
            Message::SettingsWindowOpened(_id) => Task::none(),
            Message::OpenDownloads => handle_open_downloads(self),
            Message::DownloadsWindowOpened(_id) => Task::none(),
            Message::OpenAssets => handle_open_assets(self),
            Message::AssetsWindowOpened(_id) => Task::none(),
            Message::LoadAssets => handle_load_assets(self),
            Message::AssetsLoaded(result) => handle_assets_loaded(self, result),
            Message::ToggleAssetSelection(symbol) => handle_toggle_asset_selection(self, symbol),
            Message::SelectAssetFromHeader(symbol) => handle_select_asset_from_header(self, symbol),
            Message::AssetSeriesCreated(symbol, interval, result) => {
                handle_asset_series_created(self, symbol, interval, result)
            }
            Message::WindowClosed(id) => handle_window_closed(self, id),
            
            // === Gestion de la configuration des providers ===
            Message::OpenProviderConfig => handle_open_provider_config(self),
            Message::ProviderConfigWindowOpened(_id) => Task::none(),
            Message::UpdateProviderToken(provider_type, token) => {
                handle_update_provider_token(self, provider_type, token)
            }
            Message::UpdateProviderSecret(provider_type, secret) => {
                handle_update_provider_secret(self, provider_type, secret)
            }
            Message::ApplyProviderConfig => handle_apply_provider_config(self),
            Message::SelectProvider(provider_type) => handle_select_provider(self, provider_type),
            Message::CancelProviderConfig => handle_cancel_provider_config(self),
            Message::OpenBinanceAPIKeys => handle_open_binance_api_keys(),
            
            // === Gestion des settings ===
            Message::SelectColor(field_index, color) => {
                handle_select_color(self, field_index, color)
            }
            Message::ApplySettings => handle_apply_settings(self),
            Message::CancelSettings => handle_cancel_settings(self),
            Message::ToggleColorPicker(index) => handle_toggle_color_picker(self, index),
            Message::ToggleAutoScroll => handle_toggle_auto_scroll(self),
            
            // === Messages temps réel ===
            Message::CompleteMissingData => {
                self.complete_missing_data()
            }
            Message::CompleteMissingDataComplete(results) => {
                handle_complete_missing_data_complete(self, results)
            }
            Message::LoadFullHistory(series_id) => {
                crate::app::realtime::load_full_history(self, series_id)
            }
            Message::LoadFullHistoryComplete(series_id, series_name, result) => {
                handle_load_full_history_complete(self, series_id, series_name, result)
            }
            Message::StartBatchDownload(series_id, gaps, estimated_total) => {
                handle_start_batch_download(self, series_id, gaps, estimated_total)
            }
            Message::BatchDownloadResult(series_id, candles, count, estimated, next_end) => {
                handle_batch_download_result(self, series_id, candles, count, estimated, next_end)
            }
            Message::DownloadComplete(series_id) => {
                handle_download_complete(self, series_id)
            }
            Message::PauseDownload(series_id) => {
                handle_pause_download(self, series_id)
            }
            Message::ResumeDownload(series_id) => {
                handle_resume_download(self, series_id)
            }
            Message::StopDownload(series_id) => {
                handle_stop_download(self, series_id)
            }
            Message::CompleteGaps => {
                self.complete_gaps()
            }
            Message::CompleteGapsComplete(results) => {
                handle_complete_gaps_complete(self, results)
            }
            Message::SaveSeriesComplete(results) => {
                handle_save_series_complete(self, results)
            }
            Message::RealtimeUpdate => {
                handle_realtime_update(self)
            }
            Message::RealtimeUpdateComplete(results) => {
                handle_realtime_update_complete(self, results)
            }
            
            // === Gestion des panneaux latéraux ===
            Message::ToggleVolumePanel => handle_toggle_volume_panel(self),
            Message::ToggleRSIPanel => handle_toggle_rsi_panel(self),
            Message::ToggleMACDPanel => handle_toggle_macd_panel(self),
            Message::ToggleBollingerBands => handle_toggle_bollinger_bands(self),
            Message::ToggleMovingAverage => handle_toggle_moving_average(self),
            Message::UpdateRSIPeriod(period) => handle_update_rsi_period(self, period),
            Message::UpdateRSIMethod(method) => handle_update_rsi_method(self, method),
            Message::UpdateMACDFastPeriod(period) => handle_update_macd_fast_period(self, period),
            Message::UpdateMACDSlowPeriod(period) => handle_update_macd_slow_period(self, period),
            Message::UpdateMACDSignalPeriod(period) => handle_update_macd_signal_period(self, period),
            Message::UpdateBollingerPeriod(period) => handle_update_bollinger_period(self, period),
            Message::UpdateBollingerStdDev(std_dev) => handle_update_bollinger_std_dev(self, std_dev),
            Message::UpdateMAPeriod(period) => handle_update_ma_period(self, period),
            Message::StartResizeRightPanel(pos) => handle_start_resize_right_panel(self, pos),
            Message::StartResizeBottomPanel(pos) => handle_start_resize_bottom_panel(self, pos),
            Message::UpdateResizeRightPanel(pos) => handle_update_resize_right_panel(self, pos),
            Message::UpdateResizeBottomPanel(pos) => handle_update_resize_bottom_panel(self, pos),
            Message::EndResizeRightPanel => handle_end_resize_right_panel(self),
            Message::EndResizeBottomPanel => handle_end_resize_bottom_panel(self),
            Message::StartResizeVolumePanel(pos) => handle_start_resize_volume_panel(self, pos),
            Message::UpdateResizeVolumePanel(pos) => handle_update_resize_volume_panel(self, pos),
            Message::EndResizeVolumePanel => handle_end_resize_volume_panel(self),
            Message::StartResizeRSIPanel(pos) => handle_start_resize_rsi_panel(self, pos),
            Message::StartResizeMACDPanel(pos) => handle_start_resize_macd_panel(self, pos),
            Message::UpdateResizeRSIPanel(pos) => handle_update_resize_rsi_panel(self, pos),
            Message::UpdateResizeMACDPanel(pos) => handle_update_resize_macd_panel(self, pos),
            Message::EndResizeRSIPanel => handle_end_resize_rsi_panel(self),
            Message::EndResizeMACDPanel => handle_end_resize_macd_panel(self),
            Message::SelectBottomSection(section) => handle_select_bottom_section(self, section),
            Message::SelectRightSection(section) => handle_select_right_section(self, section),
            Message::OpenSectionContextMenu(section, position) => {
                handle_open_section_context_menu(self, section, position)
            }
            Message::CloseSectionContextMenu => handle_close_section_context_menu(self),
            Message::MoveSectionToRightPanel(section) => {
                handle_move_section_to_right_panel(self, section)
            }
            Message::MoveSectionToBottomPanel(section) => {
                handle_move_section_to_bottom_panel(self, section)
            }
            Message::UpdateOrderQuantity(quantity) => handle_update_order_quantity(self, quantity),
            Message::UpdateOrderType(order_type) => handle_update_order_type(self, order_type),
            Message::UpdateLimitPrice(price) => handle_update_limit_price(self, price),
            Message::UpdateTakeProfit(tp) => handle_update_take_profit(self, tp),
            Message::UpdateStopLoss(sl) => handle_update_stop_loss(self, sl),
            Message::ToggleTPSLEnabled => handle_toggle_tp_sl_enabled(self),
            Message::PlaceBuyOrder => handle_place_buy_order(self),
            Message::PlaceSellOrder => handle_place_sell_order(self),
            Message::BuyOrderPlaced(result) => handle_buy_order_placed(self, result),
            Message::SellOrderPlaced(result) => handle_sell_order_placed(self, result),
            Message::SetRightPanelFocus(focused) => handle_set_right_panel_focus(self, focused),
            Message::SetBottomPanelFocus(focused) => handle_set_bottom_panel_focus(self, focused),
            Message::SetVolumePanelFocus(focused) => handle_set_volume_panel_focus(self, focused),
            Message::SetRSIPanelFocus(focused) => handle_set_rsi_panel_focus(self, focused),
            Message::SetMACDPanelFocus(focused) => handle_set_macd_panel_focus(self, focused),
            Message::ClearPanelFocus => handle_clear_panel_focus(self),
            Message::ToggleAccountType => handle_toggle_account_type(self),
            Message::TestProviderConnection => handle_test_provider_connection(self),
            Message::ProviderConnectionTestComplete(result) => {
                handle_provider_connection_test_complete(self, result)
            }
            Message::AccountInfoFetched(result) => {
                handle_account_info_fetched(self, result)
            }
            // === Gestion des stratégies de trading automatisées ===
            Message::RegisterRSIStrategy => handle_register_rsi_strategy(self),
            Message::RegisterMACrossoverStrategy => handle_register_ma_crossover_strategy(self),
            Message::EnableStrategy(id) => handle_enable_strategy(self, id),
            Message::DisableStrategy(id) => handle_disable_strategy(self, id),
            Message::RemoveStrategy(id) => handle_remove_strategy(self, id),
            Message::ToggleStrategyConfig(strategy_id) => {
                handle_toggle_strategy_config(self, strategy_id)
            }
            Message::UpdateStrategyParamInput { strategy_id, param_name, value } => {
                handle_update_strategy_param_input(self, strategy_id, param_name, value)
            }
            Message::ToggleStrategyTimeframe { strategy_id, timeframe } => {
                handle_toggle_strategy_timeframe(self, strategy_id, timeframe)
            }
            Message::UpdateStrategyTradingMode { strategy_id, trading_mode } => {
                handle_update_strategy_trading_mode(self, strategy_id, trading_mode)
            }
            Message::ApplyStrategyConfig(strategy_id) => {
                handle_apply_strategy_config(self, strategy_id)
            }
            Message::CancelStrategyConfig(strategy_id) => {
                handle_cancel_strategy_config(self, strategy_id)
            }
            
            // === Gestion du backtest ===
            Message::ToggleBacktestEnabled => handle_toggle_backtest_enabled(self),
            Message::SelectBacktestStrategy(strategy_id) => handle_select_backtest_strategy(self, strategy_id),
            Message::SelectBacktestDate(timestamp) => handle_select_backtest_date(self, timestamp),
            Message::SetPlayheadMode => handle_set_playhead_mode(self),
            Message::StartDragPlayhead(position) => handle_start_drag_playhead(self, position),
            Message::UpdateDragPlayhead(position) => handle_update_drag_playhead(self, position),
            Message::EndDragPlayhead => handle_end_drag_playhead(self),
            Message::StartBacktest => handle_start_backtest(self),
            Message::PauseBacktest => handle_pause_backtest(self),
            Message::StopBacktest => handle_stop_backtest(self),
            Message::BacktestTick => handle_backtest_tick(self),
        }
    }

    /// Génère la vue de l'application pour une fenêtre donnée
    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        use crate::app::views;
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => views::view_settings(self),
            Some(WindowType::ProviderConfig) => views::view_provider_config(self),
            Some(WindowType::Downloads) => views::view_downloads(self),
            Some(WindowType::Assets) => views::view_assets(self),
            Some(WindowType::Main) | None => views::view_main(self),
        }
    }

    /// Complète les données manquantes depuis Binance pour toutes les séries
    /// 
    /// Utilise Iced Tasks pour faire les requêtes en parallèle sans bloquer le thread principal.
    pub fn complete_missing_data(&mut self) -> Task<Message> {
        crate::app::realtime::complete_missing_data(self)
    }
    
    /// Applique les résultats de la complétion des données manquantes
    pub fn apply_complete_missing_data_results(&mut self, results: Vec<(SeriesId, String, Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_missing_data_results(self, results)
    }

    /// Détecte et complète les gaps dans toutes les séries de manière asynchrone
    pub fn complete_gaps(&mut self) -> Task<Message> {
        crate::app::realtime::complete_gaps(self)
    }
    
    /// Applique les résultats de la complétion des gaps
    pub fn apply_complete_gaps_results(&mut self, results: Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_gaps_results(self, results)
    }
    
    /// Met à jour les données en temps réel pour les séries actives
    pub fn update_realtime(&mut self) -> Task<Message> {
        crate::app::realtime::update_realtime(self)
    }
    
    /// Applique les résultats des mises à jour en temps réel
    pub fn apply_realtime_updates(&mut self, results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>) {
        crate::app::realtime::apply_realtime_updates(self, results)
    }
    
    /// Met à jour les informations du compte basées sur les trades
    pub fn update_account_info(&mut self) {
        let symbol = self.chart_state.series_manager
            .active_series()
            .next()
            .map(|s| s.symbol.clone())
            .unwrap_or_else(|| String::from("UNKNOWN"));
        
        let current_price = self.chart_state.series_manager
            .active_series()
            .next()
            .and_then(|s| s.data.last_candle().map(|c| c.close))
            .unwrap_or(0.0);
        
        let trade_history = &self.trading_state.trade_history;
        let total_margin_used = trade_history.total_margin_used(&symbol);
        let total_unrealized_pnl = trade_history.total_unrealized_pnl(&symbol, current_price);
        let total_realized_pnl = trade_history.total_realized_pnl();
        let open_positions_count = trade_history.open_positions_count();
        
        self.account_info.update_from_trades(
            total_margin_used,
            total_unrealized_pnl,
            total_realized_pnl,
            open_positions_count,
        );
    }
    
    /// Sauvegarde l'état complet des panneaux
    pub fn save_panel_state(&self) {
        use crate::app::persistence::PanelPersistenceState;
        let state = PanelPersistenceState {
            panels: self.ui.panels.clone(),
            active_bottom_section: self.ui.bottom_panel_sections.active_bottom_section,
            active_right_section: self.ui.bottom_panel_sections.active_right_section,
            right_panel_sections: self.ui.bottom_panel_sections.right_panel_sections.clone(),
        };
        if let Err(e) = state.save_to_file("panel_state.json") {
            eprintln!("⚠️ Erreur sauvegarde état panneaux: {}", e);
        }
    }
}

