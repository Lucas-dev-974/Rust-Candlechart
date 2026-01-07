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
    
    /// Met à jour les informations du compte après un trade
    pub fn update_from_trades(&mut self, total_margin_used: f64, total_unrealized_pnl: f64, total_realized_pnl: f64, open_positions_count: u32) {
        self.used_margin = total_margin_used;
        self.unrealized_pnl = total_unrealized_pnl;
        self.realized_pnl = total_realized_pnl;
        self.open_positions = open_positions_count;
        
        // Mettre à jour l'équité (solde + P&L non réalisé)
        self.equity = self.total_balance + self.unrealized_pnl;
        
        // Mettre à jour la marge libre (solde - marge utilisée)
        self.free_margin = self.total_balance - self.used_margin;
        
        // Mettre à jour le niveau de marge
        if self.used_margin > 0.0 {
            self.margin_level = (self.equity / self.used_margin) * 100.0;
        } else {
            self.margin_level = 0.0;
        }
        
        // Vérifier les conditions de marge
        self.margin_call = self.margin_level < 100.0 && self.margin_level > 0.0;
        self.liquidation = self.margin_level <= 0.0;
        
        // Mettre à jour le solde total avec le P&L réalisé
        self.total_balance = 10000.0 + self.realized_pnl;
        
        // Recalculer la marge libre avec le nouveau solde
        self.free_margin = self.total_balance - self.used_margin;
    }
    
    /// Met à jour les informations du compte depuis les données Binance
    /// 
    /// Les données Binance contiennent les balances pour chaque asset.
    /// On cherche la balance USDT pour le solde total.
    pub fn update_from_binance(&mut self, balances: Vec<BinanceAccountBalance>) {
        // Trouver la balance USDT
        let usdt_balance = balances.iter()
            .find(|b| b.asset == "USDT")
            .map(|b| b.free.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        
        // Mettre à jour le solde total avec la balance USDT disponible
        self.total_balance = usdt_balance;
        
        // La marge utilisée peut être calculée à partir de la balance verrouillée
        // ou on peut la garder telle quelle si elle est déjà calculée depuis les trades
        // Pour l'instant, on garde la marge utilisée existante
        
        // Recalculer les autres valeurs
        self.equity = self.total_balance + self.unrealized_pnl;
        self.free_margin = self.total_balance - self.used_margin;
        
        // Mettre à jour le niveau de marge
        if self.used_margin > 0.0 {
            self.margin_level = (self.equity / self.used_margin) * 100.0;
        } else {
            self.margin_level = 0.0;
        }
        
        // Vérifier les conditions de marge
        self.margin_call = self.margin_level < 100.0 && self.margin_level > 0.0;
        self.liquidation = self.margin_level <= 0.0;
    }
}

use crate::finance_chart::providers::binance::BinanceAccountBalance;

