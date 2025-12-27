//! Informations du compte de trading
//!
//! Ce module définit la structure pour stocker et afficher les informations
//! d'un compte de trading (solde, marge, positions, etc.)

/// Informations du compte de trading
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// Solde total du compte
    pub total_balance: f64,
    /// Solde utilisé en marge
    pub used_margin: f64,
    /// Marge libre (disponible pour ouvrir de nouvelles positions)
    pub free_margin: f64,
    /// Équité (solde + P&L non réalisé)
    pub equity: f64,
    /// P&L non réalisé (profit/perte sur positions ouvertes)
    pub unrealized_pnl: f64,
    /// P&L réalisé (profit/perte sur positions fermées)
    pub realized_pnl: f64,
    /// Nombre de positions ouvertes
    pub open_positions: u32,
    /// Effet de levier (ex: 10x, 20x, 100x)
    pub leverage: u32,
    /// Niveau de marge en pourcentage (Equity / Used Margin * 100)
    pub margin_level: f64,
    /// Indique si le compte est en appel de marge
    pub margin_call: bool,
    /// Indique si le compte est en liquidation
    pub liquidation: bool,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            total_balance: 10000.0,
            used_margin: 0.0,
            free_margin: 10000.0,
            equity: 10000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            open_positions: 0,
            leverage: 1,
            margin_level: 0.0,
            margin_call: false,
            liquidation: false,
        }
    }
}

impl AccountInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

