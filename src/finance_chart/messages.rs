//! Messages pour les interactions du graphique
//! 
//! Architecture Elm : les widgets émettent des messages,
//! l'application centrale gère les mutations d'état.

use iced::Point;
use super::tools::EditMode;

/// Messages émis par le canvas principal du graphique
#[derive(Debug, Clone)]
pub enum ChartMessage {
    // === Navigation ===
    /// Démarrer un pan (drag)
    StartPan { position: Point, time: Option<i64> },
    /// Mettre à jour le pan en cours
    UpdatePan { position: Point },
    /// Mettre à jour le pan horizontal uniquement (pour les indicateurs)
    UpdatePanHorizontal { position: Point },
    /// Terminer le pan
    EndPan,
    /// Zoom horizontal (molette)
    ZoomHorizontal { factor: f64 },
    /// Zoom vertical (ALT + molette)
    ZoomVertical { factor: f64 },
    /// Zoom les deux axes (CTRL + molette)
    ZoomBoth { factor: f64 },

    // === Dessin ===
    /// Démarrer le dessin d'un rectangle
    StartDrawingRectangle { screen_x: f32, screen_y: f32, time: i64, price: f64 },
    /// Mettre à jour l'aperçu du dessin
    UpdateDrawing { screen_x: f32, screen_y: f32 },
    /// Terminer le dessin d'un rectangle
    FinishDrawingRectangle { end_time: i64, end_price: f64 },
    /// Démarrer le dessin d'une ligne horizontale
    StartDrawingHLine { screen_y: f32, price: f64 },
    /// Terminer le dessin d'une ligne horizontale
    FinishDrawingHLine,
    /// Annuler le dessin en cours
    CancelDrawing,

    // === Édition de rectangles ===
    /// Sélectionner et commencer l'édition d'un rectangle
    StartRectangleEdit { index: usize, mode: EditMode, time: i64, price: f64 },
    /// Mettre à jour l'édition du rectangle
    UpdateRectangleEdit { time: i64, price: f64 },
    /// Terminer l'édition du rectangle
    FinishRectangleEdit,
    /// Désélectionner le rectangle
    DeselectRectangle,

    // === Édition de lignes horizontales ===
    /// Sélectionner et commencer l'édition d'une ligne
    StartHLineEdit { index: usize, price: f64 },
    /// Mettre à jour l'édition de la ligne
    UpdateHLineEdit { price: f64 },
    /// Terminer l'édition de la ligne
    FinishHLineEdit,
    /// Désélectionner la ligne
    DeselectHLine,

    // === Suppression ===
    /// Supprimer l'élément sélectionné
    DeleteSelected,

    // === Historique ===
    /// Annuler la dernière action
    Undo,
    /// Rétablir la dernière action annulée
    Redo,

    // === Persistance ===
    /// Sauvegarder les dessins
    SaveDrawings,
    /// Charger les dessins
    LoadDrawings,

    // === Position souris ===
    /// Mise à jour de la position de la souris
    MouseMoved { position: Point },
    
    // === Resize ===
    /// Mise à jour de la taille du viewport (et des bounds pour convertir positions absolues en relatives)
    Resize { width: f32, height: f32, x: f32, y: f32 },
    
    // === Backtest ===
    /// Sélectionner une date de départ pour le backtest (clic sur le graphique)
    SelectBacktestDate { time: i64 },
}

/// Messages émis par l'axe Y
#[derive(Debug, Clone)]
pub enum YAxisMessage {
    /// Zoom vertical par drag
    ZoomVertical { factor: f64 },
}

/// Messages émis par l'axe X
#[derive(Debug, Clone)]
pub enum XAxisMessage {
    /// Zoom horizontal par drag
    ZoomHorizontal { factor: f64 },
}

/// Messages émis par le panel d'outils
#[derive(Debug, Clone)]
pub enum ToolsPanelMessage {
    /// Sélectionner/désélectionner un outil
    ToggleTool { tool: super::tools::Tool },
    /// Ouvrir/fermer l'onglet d'indicateurs
    ToggleIndicatorsPanel,
}

/// Messages émis par le panel de séries
#[derive(Debug, Clone)]
pub enum SeriesPanelMessage {
    /// Sélectionner une série par son nom (depuis le select box)
    SelectSeriesByName { series_name: String },
}

