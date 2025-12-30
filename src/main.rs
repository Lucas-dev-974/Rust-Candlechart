mod finance_chart;
mod app;

use iced::{Task, Size, window, exit, Element};
use std::sync::Arc;
use std::collections::HashSet;
use finance_chart::{
    YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage,
    BinanceProvider,
    core::{SeriesId, Candle},
    ProviderType,
    settings::color_fields,
};

// Utiliser les constantes du module app::utils::constants
use app::utils::constants::{SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT};

fn main() -> iced::Result {
    iced::daemon(ChartApp::new, ChartApp::update, ChartApp::view)
        .title(ChartApp::title)
        .theme(ChartApp::theme)
        .subscription(ChartApp::subscription)
        .run()
}

// Utiliser ChartApp et Message du module app
use app::{ChartApp, Message, window_manager::WindowType, state::AccountType};

impl ChartApp {

    fn update(&mut self, message: Message) -> Task<Message> {
        use crate::app::handlers::*;
        
        match message {
            // === Gestion des messages du graphique ===
            Message::Chart(chart_msg) => {
                handle_chart_message(self, chart_msg);
                Task::none()
            }
            
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
            Message::WindowClosed(id) => handle_window_closed(self, id),
            
            // === Gestion de la configuration des providers ===
            Message::OpenProviderConfig => handle_open_provider_config(self),
            Message::ProviderConfigWindowOpened(_id) => Task::none(),
            Message::UpdateProviderToken(provider_type, token) => {
                handle_update_provider_token(self, provider_type, token)
            }
            Message::ApplyProviderConfig => handle_apply_provider_config(self),
            Message::SelectProvider(provider_type) => handle_select_provider(self, provider_type),
            Message::CancelProviderConfig => handle_cancel_provider_config(self),
            
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
            Message::UpdateDragPosition(position) => handle_update_drag_position(self, position),
            Message::EndDragSection => handle_end_drag_section(self),
            
            // === Gestion des stratégies de trading automatisées ===
            Message::RegisterRSIStrategy => handle_register_rsi_strategy(self),
            Message::RegisterMACrossoverStrategy => handle_register_ma_crossover_strategy(self),
            Message::EnableStrategy(id) => handle_enable_strategy(self, id),
            Message::DisableStrategy(id) => handle_disable_strategy(self, id),
            Message::RemoveStrategy(id) => handle_remove_strategy(self, id),
            Message::UpdateStrategyParameter { strategy_id, param_name, value } => {
                handle_update_strategy_parameter(self, strategy_id, param_name, value)
            }
            Message::UpdateStrategyTimeframes { strategy_id, timeframes } => {
                handle_update_strategy_timeframes(self, strategy_id, timeframes)
            }
            Message::ExecuteStrategies => execute_strategies(self),
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
        }
    }
    
    /// Complète les données manquantes depuis Binance pour toutes les séries
    /// 
    /// Utilise Iced Tasks pour faire les requêtes en parallèle sans bloquer le thread principal.
    fn complete_missing_data(&mut self) -> Task<Message> {
        crate::app::realtime::complete_missing_data(self)
    }
    
    /// Applique les résultats de la complétion des données manquantes
    fn apply_complete_missing_data_results(&mut self, results: Vec<(SeriesId, String, Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_missing_data_results(self, results)
    }

    /// Détecte et complète les gaps dans toutes les séries de manière asynchrone
    fn complete_gaps(&mut self) -> Task<Message> {
        crate::app::realtime::complete_gaps(self)
    }
    
    /// Applique les résultats de la complétion des gaps
    fn apply_complete_gaps_results(&mut self, results: Vec<(SeriesId, String, (i64, i64), Result<Vec<Candle>, String>)>) -> Task<Message> {
        crate::app::realtime::apply_complete_gaps_results(self, results)
    }
    
    /// Met à jour les données en temps réel pour les séries actives
    fn update_realtime(&mut self) -> Task<Message> {
        crate::app::realtime::update_realtime(self)
    }
    
    /// Applique les résultats des mises à jour en temps réel
    fn apply_realtime_updates(&mut self, results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>) {
        crate::app::realtime::apply_realtime_updates(self, results)
    }
    
    /// Met à jour les informations du compte basées sur les trades
    fn update_account_info(&mut self) {
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
    fn save_panel_state(&self) {
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

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        use crate::app::views;
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => views::view_settings(self),
            Some(WindowType::ProviderConfig) => views::view_provider_config(self),
            Some(WindowType::Downloads) => views::view_downloads(self),
            Some(WindowType::Main) | None => views::view_main(self),
        }
    }
}
