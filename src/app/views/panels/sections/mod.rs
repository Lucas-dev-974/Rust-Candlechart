//! Sections du panneau du bas

mod overview;
mod logs;
mod indicators;
mod orders;
mod trade_history;
mod account;
mod strategies;
mod backtest;

use iced::Element;
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    state::BottomPanelSection,
};

/// Affiche le contenu d'une section spécifique
pub fn view_section_content(app: &ChartApp, section: BottomPanelSection) -> Element<'_, Message> {
    match section {
        BottomPanelSection::Overview => overview::view_overview(app),
        BottomPanelSection::Logs => logs::view_logs(app),
        BottomPanelSection::Indicators => indicators::view_indicators(app),
        BottomPanelSection::Orders => orders::view_orders(app),
        BottomPanelSection::Account => account::view_account(app),
        BottomPanelSection::TradeHistory => trade_history::view_trade_history(app),
        BottomPanelSection::Strategies => strategies::view_strategies(app),
        BottomPanelSection::Backtest => backtest::view_backtest(app),
    }
}

