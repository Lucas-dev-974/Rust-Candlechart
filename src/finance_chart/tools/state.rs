//! Canvas Tools - État et types pour la barre d'outils

use iced::Color;
use serde::{Deserialize, Serialize};

/// Taille des poignées de redimensionnement
pub const HANDLE_SIZE: f32 = 8.0;
/// Taille maximale de l'historique
pub const MAX_HISTORY_SIZE: usize = 50;

/// Outils disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Rectangle,
    HorizontalLine,
}

/// Mode d'édition d'un rectangle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Move,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
    ResizeTop,
    ResizeBottom,
    ResizeLeft,
    ResizeRight,
}

/// Rectangle dessiné sur le graphique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawnRectangle {
    pub start_time: i64,
    pub start_price: f64,
    pub end_time: i64,
    pub end_price: f64,
    #[serde(with = "color_serde")]
    pub color: Color,
}

impl DrawnRectangle {
    pub fn new(start_time: i64, start_price: f64, end_time: i64, end_price: f64) -> Self {
        Self {
            start_time,
            start_price,
            end_time,
            end_price,
            color: Color::from_rgba(0.2, 0.6, 1.0, 0.3),
        }
    }
}

/// Ligne horizontale dessinée sur le graphique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawnHorizontalLine {
    pub price: f64,
    #[serde(with = "color_serde")]
    pub color: Color,
    pub width: f32,
    pub dashed: bool,
}

impl DrawnHorizontalLine {
    pub fn new(price: f64) -> Self {
        Self {
            price,
            color: Color::from_rgba(1.0, 0.8, 0.0, 0.8),
            width: 1.5,
            dashed: true,
        }
    }
}

mod color_serde {
    use iced::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct ColorData {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ColorData {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = ColorData::deserialize(deserializer)?;
        Ok(Color::from_rgba(data.r, data.g, data.b, data.a))
    }
}

/// Action enregistrée dans l'historique
#[derive(Debug, Clone)]
pub enum Action {
    CreateRectangle { rect: DrawnRectangle },
    DeleteRectangle { index: usize, rect: DrawnRectangle },
    ModifyRectangle { index: usize, old_rect: DrawnRectangle, new_rect: DrawnRectangle },
    CreateHLine { line: DrawnHorizontalLine },
    DeleteHLine { index: usize, line: DrawnHorizontalLine },
    ModifyHLine { index: usize, old_line: DrawnHorizontalLine, new_line: DrawnHorizontalLine },
}

/// Gestionnaire d'historique pour undo/redo
#[derive(Debug, Clone, Default)]
pub struct History {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl History {
    pub fn record(&mut self, action: Action) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(
        &mut self,
        rectangles: &mut Vec<DrawnRectangle>,
        hlines: &mut Vec<DrawnHorizontalLine>,
    ) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match &action {
                Action::CreateRectangle { .. } => { rectangles.pop(); }
                Action::DeleteRectangle { index, rect } => {
                    let idx = (*index).min(rectangles.len());
                    rectangles.insert(idx, rect.clone());
                }
                Action::ModifyRectangle { index, old_rect, .. } => {
                    if *index < rectangles.len() {
                        rectangles[*index] = old_rect.clone();
                    }
                }
                Action::CreateHLine { .. } => { hlines.pop(); }
                Action::DeleteHLine { index, line } => {
                    let idx = (*index).min(hlines.len());
                    hlines.insert(idx, line.clone());
                }
                Action::ModifyHLine { index, old_line, .. } => {
                    if *index < hlines.len() {
                        hlines[*index] = old_line.clone();
                    }
                }
            }
            self.redo_stack.push(action);
            true
        } else {
            false
        }
    }

    pub fn redo(
        &mut self,
        rectangles: &mut Vec<DrawnRectangle>,
        hlines: &mut Vec<DrawnHorizontalLine>,
    ) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match &action {
                Action::CreateRectangle { rect } => { rectangles.push(rect.clone()); }
                Action::DeleteRectangle { index, .. } => {
                    let idx = (*index).min(rectangles.len().saturating_sub(1));
                    if idx < rectangles.len() {
                        rectangles.remove(idx);
                    }
                }
                Action::ModifyRectangle { index, new_rect, .. } => {
                    if *index < rectangles.len() {
                        rectangles[*index] = new_rect.clone();
                    }
                }
                Action::CreateHLine { line } => { hlines.push(line.clone()); }
                Action::DeleteHLine { index, .. } => {
                    let idx = (*index).min(hlines.len().saturating_sub(1));
                    if idx < hlines.len() {
                        hlines.remove(idx);
                    }
                }
                Action::ModifyHLine { index, new_line, .. } => {
                    if *index < hlines.len() {
                        hlines[*index] = new_line.clone();
                    }
                }
            }
            self.undo_stack.push(action);
            true
        } else {
            false
        }
    }
}

/// État d'édition d'un rectangle
#[derive(Debug, Clone, Default)]
pub struct EditState {
    pub selected_index: Option<usize>,
    pub edit_mode: Option<EditMode>,
    pub is_editing: bool,
    pub start_time: Option<i64>,
    pub start_price: Option<f64>,
    pub original_rect: Option<DrawnRectangle>,
}

impl EditState {
    pub fn start(&mut self, index: usize, mode: EditMode, time: i64, price: f64, rect: DrawnRectangle) {
        self.selected_index = Some(index);
        self.edit_mode = Some(mode);
        self.is_editing = true;
        self.start_time = Some(time);
        self.start_price = Some(price);
        self.original_rect = Some(rect);
    }

    pub fn finish(&mut self) {
        self.is_editing = false;
        self.edit_mode = None;
        self.start_time = None;
        self.start_price = None;
        self.original_rect = None;
    }

    pub fn deselect(&mut self) {
        self.selected_index = None;
        self.finish();
    }
}

/// État d'édition d'une ligne horizontale
#[derive(Debug, Clone, Default)]
pub struct HLineEditState {
    pub selected_index: Option<usize>,
    pub is_editing: bool,
    pub start_price: Option<f64>,
    pub original_line: Option<DrawnHorizontalLine>,
}

impl HLineEditState {
    pub fn start(&mut self, index: usize, price: f64, line: DrawnHorizontalLine) {
        self.selected_index = Some(index);
        self.is_editing = true;
        self.start_price = Some(price);
        self.original_line = Some(line);
    }

    pub fn finish(&mut self) {
        self.is_editing = false;
        self.start_price = None;
        self.original_line = None;
    }

    pub fn deselect(&mut self) {
        self.selected_index = None;
        self.finish();
    }
}

/// État de dessin en cours
#[derive(Debug, Clone, Default)]
pub struct DrawingState {
    pub is_drawing: bool,
    pub start_screen_point: Option<(f32, f32)>,
    pub start_time: Option<i64>,
    pub start_price: Option<f64>,
    pub current_screen_point: Option<(f32, f32)>,
}

impl DrawingState {
    pub fn start(&mut self, screen_x: f32, screen_y: f32, time: i64, price: f64) {
        self.is_drawing = true;
        self.start_screen_point = Some((screen_x, screen_y));
        self.start_time = Some(time);
        self.start_price = Some(price);
        self.current_screen_point = Some((screen_x, screen_y));
    }

    pub fn update(&mut self, screen_x: f32, screen_y: f32) {
        self.current_screen_point = Some((screen_x, screen_y));
    }

    pub fn finish(&mut self, end_time: i64, end_price: f64) -> Option<DrawnRectangle> {
        if self.is_drawing {
            if let (Some(start_time), Some(start_price)) = (self.start_time, self.start_price) {
                let rect = DrawnRectangle::new(start_time, start_price, end_time, end_price);
                self.reset();
                return Some(rect);
            }
        }
        self.reset();
        None
    }

    pub fn finish_hline(&mut self) -> Option<DrawnHorizontalLine> {
        if self.is_drawing {
            if let Some(price) = self.start_price {
                let line = DrawnHorizontalLine::new(price);
                self.reset();
                return Some(line);
            }
        }
        self.reset();
        None
    }

    pub fn cancel(&mut self) {
        self.reset();
    }

    fn reset(&mut self) {
        self.is_drawing = false;
        self.start_screen_point = None;
        self.start_time = None;
        self.start_price = None;
        self.current_screen_point = None;
    }
}

/// État partagé du panel d'outils
#[derive(Debug, Clone, Default)]
pub struct ToolsState {
    pub selected_tool: Option<Tool>,
    pub rectangles: Vec<DrawnRectangle>,
    pub horizontal_lines: Vec<DrawnHorizontalLine>,
    pub drawing: DrawingState,
    pub editing: EditState,
    pub hline_editing: HLineEditState,
    pub history: History,
}

/// Structure pour la sérialisation des dessins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingsData {
    pub rectangles: Vec<DrawnRectangle>,
    pub horizontal_lines: Vec<DrawnHorizontalLine>,
}

impl ToolsState {
    pub fn export_drawings(&self) -> Result<String, serde_json::Error> {
        let data = DrawingsData {
            rectangles: self.rectangles.clone(),
            horizontal_lines: self.horizontal_lines.clone(),
        };
        serde_json::to_string_pretty(&data)
    }

    pub fn import_drawings(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let data: DrawingsData = serde_json::from_str(json)?;
        self.rectangles = data.rectangles;
        self.horizontal_lines = data.horizontal_lines;
        self.history = History::default();
        Ok(())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.export_drawings()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        self.import_drawings(&json)?;
        Ok(())
    }
}

