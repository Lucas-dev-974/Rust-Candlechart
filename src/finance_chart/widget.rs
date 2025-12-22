//! Widget Canvas principal pour le graphique financier
//!
//! Architecture Elm : émet des messages pour les mutations d'état,
//! reçoit des références immuables pour le rendu.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Action as CanvasAction, Path, Text};
use iced::{Element, Event, Length, Point, Rectangle, Size, Color};
use iced::{keyboard, mouse};

use super::render::{
    render_candlesticks, render_current_price_line, render_grid,
    render_crosshair, render_tooltip, find_candle_at_position,
    draw_rectangle, draw_preview_rectangle,
    draw_horizontal_line, draw_hline_preview, hit_test_hline,
    grid::GridStyle, current_price::CurrentPriceStyle,
    crosshair::CrosshairStyle,
};
use super::interaction::{hit_test_rectangles, cursor_for_edit_mode};
use super::state::ChartState;
use super::tools_canvas::{Tool, ToolsState};
use super::settings::{SettingsState, ChartStyle};
use super::messages::ChartMessage;

/// État local du widget (UI uniquement, pas de données business)
#[derive(Debug, Clone, Default)]
pub struct WidgetState {
    /// ALT est maintenu
    pub alt_pressed: bool,
    /// CTRL est maintenu
    pub ctrl_pressed: bool,
    /// SHIFT est maintenu (pour afficher le tooltip)
    pub shift_pressed: bool,
}

/// Program Iced pour le rendu du graphique
/// Reçoit des références immuables, émet des messages pour les mutations
pub struct ChartProgram<'a> {
    chart_state: &'a ChartState,
    tools_state: &'a ToolsState,
    settings_state: &'a SettingsState,
    chart_style: &'a ChartStyle,
    /// Indique si un panneau a le focus (désactive les interactions du chart)
    panel_focused: bool,
}

impl<'a> ChartProgram<'a> {
    pub fn new(
        chart_state: &'a ChartState,
        tools_state: &'a ToolsState,
        settings_state: &'a SettingsState,
        chart_style: &'a ChartStyle,
        panel_focused: bool,
    ) -> Self {
        Self { chart_state, tools_state, settings_state, chart_style, panel_focused }
    }

    /// Génère des couleurs différentes pour chaque série
    fn get_series_colors(&self, series_idx: usize, series_id: &super::core::SeriesId) -> super::render::candlestick::CandleColors {
        use iced::Color;
        
        // Palette de couleurs pour différentes séries
        let color_palettes = [
            // Palette 1: Vert/Rouge classique
            (Color::from_rgb(0.0, 0.8, 0.0), Color::from_rgb(0.8, 0.0, 0.0), Color::from_rgb(0.5, 0.5, 0.5)),
            // Palette 2: Bleu/Orange
            (Color::from_rgb(0.2, 0.6, 1.0), Color::from_rgb(1.0, 0.6, 0.2), Color::from_rgb(0.6, 0.6, 0.6)),
            // Palette 3: Cyan/Magenta
            (Color::from_rgb(0.0, 0.8, 0.8), Color::from_rgb(0.8, 0.0, 0.8), Color::from_rgb(0.5, 0.5, 0.5)),
            // Palette 4: Jaune/Violet
            (Color::from_rgb(0.8, 0.8, 0.0), Color::from_rgb(0.6, 0.0, 0.8), Color::from_rgb(0.5, 0.5, 0.5)),
            // Palette 5: Vert clair/Rouge foncé
            (Color::from_rgb(0.4, 1.0, 0.4), Color::from_rgb(0.6, 0.0, 0.0), Color::from_rgb(0.5, 0.5, 0.5)),
        ];
        
        // Utiliser la palette par défaut si on dépasse
        let palette = color_palettes.get(series_idx % color_palettes.len())
            .copied()
            .unwrap_or(color_palettes[0]);
        
        // Vérifier si la série a une couleur personnalisée
        if let Some(series_data) = self.chart_state.series_manager.get_series(series_id) {
            if let Some(custom_color) = series_data.color {
                // Utiliser la couleur personnalisée avec des variantes pour bullish/bearish
                let bullish = Color::from_rgba(
                    custom_color.r.min(1.0),
                    custom_color.g.min(1.0),
                    custom_color.b.min(1.0),
                    custom_color.a,
                );
                let bearish = Color::from_rgba(
                    (custom_color.r * 0.7).min(1.0),
                    (custom_color.g * 0.3).min(1.0),
                    (custom_color.b * 0.3).min(1.0),
                    custom_color.a,
                );
                let wick = Color::from_rgba(
                    custom_color.r * 0.6,
                    custom_color.g * 0.6,
                    custom_color.b * 0.6,
                    custom_color.a,
                );
                return super::render::candlestick::CandleColors {
                    bullish,
                    bearish,
                    wick,
                };
            }
        }
        
        // Utiliser la palette par index
        super::render::candlestick::CandleColors {
            bullish: palette.0,
            bearish: palette.1,
            wick: palette.2,
        }
    }

    /// Dessine le label du prix actuel sur le bord droit (avant la zone Y)
    fn draw_current_price_label(&self, frame: &mut Frame, candle: &crate::finance_chart::core::Candle) {
        let viewport = &self.chart_state.viewport;
        let current_price = candle.close;
        let y = viewport.price_scale().price_to_y(current_price);
        
        // Ne dessiner que si visible
        if y < 0.0 || y > viewport.height() {
            return;
        }
        
        // Couleur selon si le prix est haussier ou baissier
        let is_bullish = candle.close >= candle.open;
        let bg_color = if is_bullish {
            Color::from_rgba(0.0, 0.5, 0.0, 1.0) // Vert foncé opaque
        } else {
            Color::from_rgba(0.5, 0.0, 0.0, 1.0) // Rouge foncé opaque
        };
        
        // Formater le prix avec 2 décimales
        let price_label = format!("{:.2}", current_price);
        
        let padding_x = 4.0;
        let padding_y = 2.0;
        let label_width = 60.0;
        let label_height = 11.0 + padding_y * 2.0;
        
        let width = viewport.width();
        let label_x = width - label_width - 2.0;
        let label_y = y - label_height / 2.0;
        
        // Fond du label
        let bg_rect = Path::rectangle(
            Point::new(label_x, label_y),
            Size::new(label_width, label_height),
        );
        frame.fill(&bg_rect, bg_color);
        
        // Texte
        let text = Text {
            content: price_label,
            position: Point::new(label_x + padding_x, label_y + padding_y),
            color: Color::WHITE,
            size: iced::Pixels(11.0),
            ..Text::default()
        };
        frame.fill_text(text);
    }

    /// Dessine tous les éléments dessinés (rectangles et lignes horizontales)
    fn draw_all_drawings(&self, frame: &mut Frame) {
        let viewport = &self.chart_state.viewport;

        // Dessiner les lignes horizontales
        let selected_hline = self.tools_state.hline_editing.selected_index;
        for (index, line) in self.tools_state.horizontal_lines.iter().enumerate() {
            let is_selected = selected_hline == Some(index);
            draw_horizontal_line(frame, viewport, line, is_selected);
        }

        // Dessiner les rectangles
        let selected_rect = self.tools_state.editing.selected_index;
        for (index, rect) in self.tools_state.rectangles.iter().enumerate() {
            let is_selected = selected_rect == Some(index);
            draw_rectangle(frame, viewport, rect, is_selected);
        }

        // Dessiner l'aperçu du rectangle en cours de dessin
        if self.tools_state.drawing.is_drawing && self.tools_state.selected_tool == Some(Tool::Rectangle) {
            if let (Some((start_x, start_y)), Some((current_x, current_y))) = (
                self.tools_state.drawing.start_screen_point,
                self.tools_state.drawing.current_screen_point,
            ) {
                draw_preview_rectangle(frame, start_x, start_y, current_x, current_y);
            }
        }

        // Dessiner l'aperçu de la ligne horizontale en cours
        if self.tools_state.drawing.is_drawing && self.tools_state.selected_tool == Some(Tool::HorizontalLine) {
            if let Some((_, y)) = self.tools_state.drawing.current_screen_point {
                draw_hline_preview(frame, y, viewport.width());
            }
        }
    }
}

impl<'a> Program<ChartMessage> for ChartProgram<'a> {
    type State = WidgetState;

    fn draw(
        &self,
        widget_state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Mettre à jour la position de la souris (lecture seule)
        let mouse_position = cursor.position_in(bounds);

        // Les couleurs sont maintenant gérées par série dans get_series_colors()
        let grid_style = GridStyle {
            line_color: self.chart_style.grid_color.to_iced(),
            line_width: 1.0,
        };
        let price_style = CurrentPriceStyle {
            line_color: self.chart_style.current_price_color.to_iced(),
            ..Default::default()
        };

        // Rendu du graphique de base
        // Fond avec la couleur personnalisée
        let bg_color = self.chart_style.background_color.to_iced();
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), bg_color);
        
        render_grid(&mut frame, &self.chart_state.viewport, Some(grid_style));
        
        // Rendre toutes les séries actives avec des couleurs différentes
        let visible_series = self.chart_state.visible_candles();
        for (series_idx, (series_id, candles)) in visible_series.iter().enumerate() {
            // Générer des couleurs différentes pour chaque série
            let series_colors = self.get_series_colors(series_idx, series_id);
            render_candlesticks(&mut frame, candles, &self.chart_state.viewport, Some(series_colors));
        }
        
        // Afficher la ligne de prix courant de la première série active
        if let Some(last_candle) = self.chart_state.last_candle() {
            render_current_price_line(&mut frame, &self.chart_state.viewport, last_candle.close, Some(price_style));
            // Afficher le label du prix actuel sur le bord droit (avant la zone Y)
            self.draw_current_price_label(&mut frame, last_candle);
        }

        // Rendu des dessins (rectangles et lignes)
        self.draw_all_drawings(&mut frame);

        // Rendu du crosshair (seulement si le dialog n'est pas ouvert)
        if !self.settings_state.is_open {
            if let Some(pos) = mouse_position {
                let crosshair_style = CrosshairStyle {
                    line_color: self.chart_style.crosshair_color.to_iced(),
                    label_text_color: self.chart_style.text_color.to_iced(),
                    ..Default::default()
                };
                render_crosshair(&mut frame, &self.chart_state.viewport, pos, Some(crosshair_style));

                // Rendu du tooltip OHLC (si SHIFT maintenu)
                if widget_state.shift_pressed {
                    let visible_series = self.chart_state.visible_candles();
                    // Chercher dans toutes les séries actives
                    for (_, candles) in visible_series.iter() {
                        if let Some(candle) = find_candle_at_position(pos.x, candles, &self.chart_state.viewport) {
                            render_tooltip(&mut frame, candle, pos, &self.chart_state.viewport, None);
                            break; // Afficher seulement le premier trouvé
                        }
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        widget_state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<CanvasAction<ChartMessage>> {
        // Vérifier si la taille a changé et émettre un message de resize
        let current_width = self.chart_state.viewport.width();
        let current_height = self.chart_state.viewport.height();
        if (current_width - bounds.width).abs() > 1.0 || (current_height - bounds.height).abs() > 1.0 {
            return Some(CanvasAction::publish(ChartMessage::Resize {
                width: bounds.width,
                height: bounds.height,
            }));
        }
        
        match event {
            // === Gestion des touches clavier ===
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                return self.handle_key_press(widget_state, key.clone());
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                return self.handle_key_release(widget_state, key.clone());
            }
            // === Gestion de la souris ===
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    return self.handle_mouse_press(position);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                return self.handle_mouse_release(cursor.position_in(bounds));
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let position = Point::new(position.x - bounds.x, position.y - bounds.y);
                return self.handle_mouse_move(position, bounds);
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                return self.handle_scroll(widget_state, *delta);
            }
            _ => {}
        }
        
        None
    }

    fn mouse_interaction(
        &self,
        _widget_state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // Vérifier si le curseur est dans le canvas du graphique
        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };

        // Édition en cours
        if self.tools_state.editing.is_editing {
            if let Some(mode) = self.tools_state.editing.edit_mode {
                return cursor_for_edit_mode(mode);
            }
        }
        if self.tools_state.hline_editing.is_editing {
            return mouse::Interaction::ResizingVertically;
        }
        
        // Survol d'un rectangle ou ligne
        // Test rectangles
        if let Some(result) = hit_test_rectangles(
            position,
            &self.tools_state.rectangles,
            self.tools_state.editing.selected_index,
            &self.chart_state.viewport,
        ) {
            return cursor_for_edit_mode(result.mode);
        }
        
        // Test lignes horizontales
        if hit_test_hline(position.y, &self.tools_state.horizontal_lines, &self.chart_state.viewport).is_some() {
            return mouse::Interaction::ResizingVertically;
        }
        
        // Curseur croix uniquement sur le canvas du graphique
        mouse::Interaction::Crosshair
    }
}

// ============================================================================
// Handlers d'événements - émettent des messages
// ============================================================================

impl<'a> ChartProgram<'a> {
    fn handle_key_press(
        &self,
        widget_state: &mut WidgetState,
        key: keyboard::Key,
    ) -> Option<CanvasAction<ChartMessage>> {
        match key {
            keyboard::Key::Named(keyboard::key::Named::Alt) => {
                widget_state.alt_pressed = true;
                Some(CanvasAction::request_redraw())
            }
            keyboard::Key::Named(keyboard::key::Named::Control) => {
                widget_state.ctrl_pressed = true;
                Some(CanvasAction::request_redraw())
            }
            keyboard::Key::Named(keyboard::key::Named::Shift) => {
                widget_state.shift_pressed = true;
                Some(CanvasAction::request_redraw())
            }
            keyboard::Key::Named(keyboard::key::Named::Delete) |
            keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                Some(CanvasAction::publish(ChartMessage::DeleteSelected))
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                // Annuler l'action en cours
                if self.tools_state.drawing.is_drawing {
                    Some(CanvasAction::publish(ChartMessage::CancelDrawing))
                } else if self.tools_state.editing.is_editing {
                    Some(CanvasAction::publish(ChartMessage::FinishRectangleEdit))
                } else if self.tools_state.hline_editing.is_editing {
                    Some(CanvasAction::publish(ChartMessage::FinishHLineEdit))
                } else if self.tools_state.editing.selected_index.is_some() {
                    Some(CanvasAction::publish(ChartMessage::DeselectRectangle))
                } else if self.tools_state.hline_editing.selected_index.is_some() {
                    Some(CanvasAction::publish(ChartMessage::DeselectHLine))
                } else {
                    Some(CanvasAction::request_redraw())
                }
            }
            keyboard::Key::Character(c) if c.as_str() == "z" && widget_state.ctrl_pressed => {
                Some(CanvasAction::publish(ChartMessage::Undo))
            }
            keyboard::Key::Character(c) if c.as_str() == "y" && widget_state.ctrl_pressed => {
                Some(CanvasAction::publish(ChartMessage::Redo))
            }
            keyboard::Key::Character(c) if c.as_str() == "s" && widget_state.ctrl_pressed => {
                Some(CanvasAction::publish(ChartMessage::SaveDrawings))
            }
            keyboard::Key::Character(c) if c.as_str() == "o" && widget_state.ctrl_pressed => {
                Some(CanvasAction::publish(ChartMessage::LoadDrawings))
            }
            _ => None
        }
    }

    fn handle_key_release(
        &self,
        widget_state: &mut WidgetState,
        key: keyboard::Key,
    ) -> Option<CanvasAction<ChartMessage>> {
        match key {
            keyboard::Key::Named(keyboard::key::Named::Alt) => {
                widget_state.alt_pressed = false;
                Some(CanvasAction::request_redraw())
            }
            keyboard::Key::Named(keyboard::key::Named::Control) => {
                widget_state.ctrl_pressed = false;
                Some(CanvasAction::request_redraw())
            }
            keyboard::Key::Named(keyboard::key::Named::Shift) => {
                widget_state.shift_pressed = false;
                Some(CanvasAction::request_redraw())
            }
            _ => None
        }
    }

    fn handle_mouse_press(&self, position: Point) -> Option<CanvasAction<ChartMessage>> {
        // Ignorer les événements si un panneau a le focus
        if self.panel_focused {
            return None;
        }
        
        let viewport = &self.chart_state.viewport;
        let time = viewport.time_scale().x_to_time(position.x);
        let price = viewport.price_scale().y_to_price(position.y);

        // Clic sur un rectangle existant
        if let Some(result) = hit_test_rectangles(
            position,
            &self.tools_state.rectangles,
            self.tools_state.editing.selected_index,
            &self.chart_state.viewport,
        ) {
            return Some(CanvasAction::publish(ChartMessage::StartRectangleEdit {
                index: result.index,
                mode: result.mode,
                time,
                price,
            }));
        }
        
        // Clic sur une ligne horizontale existante
        if let Some(index) = hit_test_hline(position.y, &self.tools_state.horizontal_lines, &self.chart_state.viewport) {
            return Some(CanvasAction::publish(ChartMessage::StartHLineEdit {
                index,
                price,
            }));
        }
        
        // Outil actif - priorité sur le pan
        match self.tools_state.selected_tool {
            Some(Tool::Rectangle) => {
                return Some(CanvasAction::publish(ChartMessage::StartDrawingRectangle {
                    screen_x: position.x,
                    screen_y: position.y,
                    time,
                    price,
                }));
            }
            Some(Tool::HorizontalLine) => {
                return Some(CanvasAction::publish(ChartMessage::StartDrawingHLine {
                    screen_y: position.y,
                    price,
                }));
            }
            None => {
                // Pas d'outil actif - démarrer le pan
                // (même si quelque chose est sélectionné, on peut toujours faire un pan)
                return Some(CanvasAction::publish(ChartMessage::StartPan { position }));
            }
        }
    }

    fn handle_mouse_release(&self, cursor_position: Option<Point>) -> Option<CanvasAction<ChartMessage>> {
        // Fin d'édition rectangle
        if self.tools_state.editing.is_editing {
            return Some(CanvasAction::publish(ChartMessage::FinishRectangleEdit));
        }
        
        // Fin d'édition ligne horizontale
        if self.tools_state.hline_editing.is_editing {
            return Some(CanvasAction::publish(ChartMessage::FinishHLineEdit));
        }
        
        // Fin de dessin
        if self.tools_state.drawing.is_drawing {
            if let Some(position) = cursor_position {
                let viewport = &self.chart_state.viewport;
                let end_time = viewport.time_scale().x_to_time(position.x);
                let end_price = viewport.price_scale().y_to_price(position.y);
                
                match self.tools_state.selected_tool {
                    Some(Tool::Rectangle) => {
                        return Some(CanvasAction::publish(ChartMessage::FinishDrawingRectangle {
                            end_time,
                            end_price,
                        }));
                    }
                    Some(Tool::HorizontalLine) => {
                        return Some(CanvasAction::publish(ChartMessage::FinishDrawingHLine));
                    }
                    None => {}
                }
            } else {
                return Some(CanvasAction::publish(ChartMessage::CancelDrawing));
            }
        }
        
        // Fin du pan
        Some(CanvasAction::publish(ChartMessage::EndPan))
    }

    fn handle_mouse_move(&self, position: Point, _bounds: Rectangle) -> Option<CanvasAction<ChartMessage>> {
        // Ignorer les événements si un panneau a le focus
        if self.panel_focused {
            return None;
        }
        
        let viewport = &self.chart_state.viewport;
        let time = viewport.time_scale().x_to_time(position.x);
        let price = viewport.price_scale().y_to_price(position.y);
        
        // Vérifier si on est en train de faire quelque chose qui bloque le pan
        let is_busy = self.tools_state.drawing.is_drawing 
            || self.tools_state.editing.is_editing 
            || self.tools_state.hline_editing.is_editing;
        
        // PRIORITÉ 1 : Pan (si actif et pas occupé par autre chose)
        if self.chart_state.interaction.is_panning && !is_busy {
            return Some(CanvasAction::publish(ChartMessage::UpdatePan { position }));
        }
        
        // PRIORITÉ 2 : Édition rectangle (si active)
        if self.tools_state.editing.is_editing {
            return Some(CanvasAction::publish(ChartMessage::UpdateRectangleEdit { time, price }));
        }
        
        // PRIORITÉ 3 : Édition ligne horizontale (si active)
        if self.tools_state.hline_editing.is_editing {
            return Some(CanvasAction::publish(ChartMessage::UpdateHLineEdit { price }));
        }
        
        // PRIORITÉ 4 : Dessin en cours
        if self.tools_state.drawing.is_drawing {
            return Some(CanvasAction::publish(ChartMessage::UpdateDrawing {
                screen_x: position.x,
                screen_y: position.y,
            }));
        }
        
        // PRIORITÉ 5 : Mise à jour position souris (pour le crosshair)
        Some(CanvasAction::publish(ChartMessage::MouseMoved { position }))
    }

    fn handle_scroll(&self, widget_state: &WidgetState, delta: mouse::ScrollDelta) -> Option<CanvasAction<ChartMessage>> {
        // Ignorer les événements si un panneau a le focus
        if self.panel_focused {
            return None;
        }
        
        let zoom_factor = match delta {
            mouse::ScrollDelta::Lines { y, .. } => if y > 0.0 { 0.9 } else { 1.1 },
            mouse::ScrollDelta::Pixels { y, .. } => if y > 0.0 { 0.95 } else { 1.05 },
        };
        
        if widget_state.ctrl_pressed {
            Some(CanvasAction::publish(ChartMessage::ZoomBoth { factor: zoom_factor }))
        } else if widget_state.alt_pressed {
            Some(CanvasAction::publish(ChartMessage::ZoomVertical { factor: zoom_factor }))
        } else {
            Some(CanvasAction::publish(ChartMessage::ZoomHorizontal { factor: zoom_factor }))
        }
    }
}

// ============================================================================
// Factory function
// ============================================================================

/// Helper pour créer un élément de graphique
/// Prend des références immuables, retourne un Element qui émet ChartMessage
pub fn chart<'a>(
    chart_state: &'a ChartState,
    tools_state: &'a ToolsState,
    settings_state: &'a SettingsState,
    chart_style: &'a ChartStyle,
    panel_focused: bool,
) -> Element<'a, ChartMessage> {
    Canvas::new(ChartProgram::new(chart_state, tools_state, settings_state, chart_style, panel_focused))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
