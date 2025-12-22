//! Gestion du type de compte (Démo/Paper vs Réel)
//!
//! Ce module définit les types de comptes disponibles et leur état.

/// Type de compte disponible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    /// Compte démo/paper (simulation, pas de transactions réelles)
    Demo,
    /// Compte réel (lié au provider actif, transactions réelles)
    Real,
}

impl AccountType {
    /// Retourne le nom d'affichage du type de compte
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Demo => "Démo (Paper Trading)",
            Self::Real => "Compte Réel",
        }
    }
    
    /// Retourne la description du type de compte
    pub fn description(&self) -> &'static str {
        match self {
            Self::Demo => "Mode simulation - Aucune transaction réelle ne sera effectuée",
            Self::Real => "Mode réel - Les transactions seront effectuées avec le provider actif",
        }
    }
}

/// État du type de compte
#[derive(Debug, Clone)]
pub struct AccountTypeState {
    /// Type de compte actuellement sélectionné
    pub account_type: AccountType,
}

impl Default for AccountTypeState {
    fn default() -> Self {
        Self {
            account_type: AccountType::Demo, // Par défaut, on commence en mode démo pour la sécurité
        }
    }
}

impl AccountTypeState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Change le type de compte
    pub fn set_account_type(&mut self, account_type: AccountType) {
        self.account_type = account_type;
    }
    
    /// Retourne true si le compte est en mode démo
    pub fn is_demo(&self) -> bool {
        self.account_type == AccountType::Demo
    }
    
    /// Retourne true si le compte est en mode réel
    pub fn is_real(&self) -> bool {
        self.account_type == AccountType::Real
    }
}

