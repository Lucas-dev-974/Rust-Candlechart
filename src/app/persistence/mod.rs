//! Persistance des données
//!
//! Ce module gère la sauvegarde et le chargement des données persistantes.

mod panel_persistence;
mod trading_persistence;
mod timeframe_persistence;
mod strategy_persistence;
mod assets_persistence;
mod selected_assets_persistence;

pub use panel_persistence::PanelPersistenceState;
pub use trading_persistence::TradingPersistenceState;
pub use timeframe_persistence::TimeframePersistenceState;
pub use strategy_persistence::{
    StrategiesPersistenceState,
    strategy_to_persistence, persistence_to_strategy,
};
pub use assets_persistence::AssetsPersistenceState;
pub use selected_assets_persistence::SelectedAssetsPersistenceState;



