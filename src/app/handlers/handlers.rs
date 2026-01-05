//! Handlers pour les messages de l'application
//!
//! Ce module contient toute la logique de gestion des messages, incluant
//! les handlers pour les messages du graphique, des outils, des settings, etc.

use crate::finance_chart::{
    ChartMessage,
    tools::Action as HistoryAction,
};
use crate::app::app_state::ChartApp;
use crate::app::state::BottomPanelSection;

/// Gère les messages du graphique
pub fn handle_chart_message(app: &mut ChartApp, msg: ChartMessage) {
    match msg {
        // === Navigation ===
        ChartMessage::StartPan { position, time } => {
            app.chart_state.start_pan(position);
            // Si un time est fourni, également gérer la sélection de date de backtest
            if let Some(time) = time {
                use crate::app::state::BottomPanelSection;
                if app.ui.bottom_panel_sections.active_bottom_section == BottomPanelSection::Backtest {
                    // Ne pas permettre de redéfinir la position si la lecture est en cours
                    if !app.ui.backtest_state.is_playing {
                        // Mettre à jour le timestamp de départ
                        app.ui.backtest_state.start_timestamp = Some(time);
                        
                        // Réinitialiser les index pour que la barre se positionne sur la nouvelle date
                        app.ui.backtest_state.current_index = 0;
                        app.ui.backtest_state.start_index = None;
                    }
                }
            }
        }
        ChartMessage::UpdatePan { position } => {
            app.chart_state.update_pan(position);
        }
        ChartMessage::UpdatePanHorizontal { position } => {
            app.chart_state.update_pan_horizontal(position);
        }
        ChartMessage::EndPan => {
            app.chart_state.end_pan();
        }
        ChartMessage::ZoomHorizontal { factor } => {
            app.chart_state.zoom(factor);
        }
        ChartMessage::ZoomVertical { factor } => {
            app.chart_state.zoom_vertical(factor);
        }
        ChartMessage::ZoomBoth { factor } => {
            app.chart_state.zoom_both(factor);
        }
        
        // === Dessin de rectangles ===
        ChartMessage::StartDrawingRectangle { screen_x, screen_y, time, price } => {
            app.tools_state.drawing.start(screen_x, screen_y, time, price);
        }
        ChartMessage::UpdateDrawing { screen_x, screen_y } => {
            app.tools_state.drawing.update(screen_x, screen_y);
        }
        ChartMessage::FinishDrawingRectangle { end_time, end_price } => {
            if let Some(rect) = app.tools_state.drawing.finish(end_time, end_price) {
                app.tools_state.history.record(HistoryAction::CreateRectangle { rect: rect.clone() });
                let new_index = app.tools_state.rectangles.len();
                app.tools_state.rectangles.push(rect);
                app.tools_state.editing.selected_index = Some(new_index);
                app.tools_state.selected_tool = None;
            }
        }
        
        // === Dessin de lignes horizontales ===
        ChartMessage::StartDrawingHLine { screen_y, price } => {
            app.tools_state.drawing.start(0.0, screen_y, 0, price);
        }
        ChartMessage::FinishDrawingHLine => {
            if let Some(line) = app.tools_state.drawing.finish_hline() {
                app.tools_state.history.record(HistoryAction::CreateHLine { line: line.clone() });
                let new_index = app.tools_state.horizontal_lines.len();
                app.tools_state.horizontal_lines.push(line);
                app.tools_state.hline_editing.selected_index = Some(new_index);
                app.tools_state.selected_tool = None;
            }
        }
        ChartMessage::CancelDrawing => {
            app.tools_state.drawing.cancel();
        }
        
        // === Édition de rectangles ===
        ChartMessage::StartRectangleEdit { index, mode, time, price } => {
            if index < app.tools_state.rectangles.len() {
                let rect_clone = app.tools_state.rectangles[index].clone();
                app.tools_state.editing.start(index, mode, time, price, rect_clone);
            }
        }
        ChartMessage::UpdateRectangleEdit { time, price } => {
            // Note: Le clone est nécessaire pour éviter un conflit d'emprunt
            // (on emprunte editing en lecture et rectangles en écriture simultanément)
            if let Some(index) = app.tools_state.editing.selected_index {
                if index < app.tools_state.rectangles.len() {
                    use crate::finance_chart::interaction::apply_edit_update;
                    let edit_state = app.tools_state.editing.clone();
                    apply_edit_update(&mut app.tools_state.rectangles[index], &edit_state, time, price);
                }
            }
        }
        ChartMessage::FinishRectangleEdit => {
            finish_rectangle_edit(app);
        }
        ChartMessage::DeselectRectangle => {
            app.tools_state.editing.deselect();
        }
        
        // === Édition de lignes horizontales ===
        ChartMessage::StartHLineEdit { index, price } => {
            if index < app.tools_state.horizontal_lines.len() {
                let line_clone = app.tools_state.horizontal_lines[index].clone();
                app.tools_state.hline_editing.start(index, price, line_clone);
            }
        }
        ChartMessage::UpdateHLineEdit { price } => {
            if let Some(index) = app.tools_state.hline_editing.selected_index {
                if index < app.tools_state.horizontal_lines.len() {
                    if let Some(ref original) = app.tools_state.hline_editing.original_line {
                        if let Some(start_price) = app.tools_state.hline_editing.start_price {
                            let delta = price - start_price;
                            app.tools_state.horizontal_lines[index].price = original.price + delta;
                        }
                    }
                }
            }
        }
        ChartMessage::FinishHLineEdit => {
            finish_hline_edit(app);
        }
        ChartMessage::DeselectHLine => {
            app.tools_state.hline_editing.deselect();
        }
        
        // === Suppression ===
        ChartMessage::DeleteSelected => {
            delete_selected(app);
        }
        
        // === Historique ===
        ChartMessage::Undo => {
            app.tools_state.editing.deselect();
            app.tools_state.hline_editing.deselect();
            app.tools_state.history.undo(
                &mut app.tools_state.rectangles,
                &mut app.tools_state.horizontal_lines,
            );
        }
        ChartMessage::Redo => {
            app.tools_state.editing.deselect();
            app.tools_state.hline_editing.deselect();
            app.tools_state.history.redo(
                &mut app.tools_state.rectangles,
                &mut app.tools_state.horizontal_lines,
            );
        }
        
        // === Persistance ===
        ChartMessage::SaveDrawings => {
            if let Err(e) = app.tools_state.save_to_file("drawings.json") {
                eprintln!("❌ Erreur de sauvegarde: {}", e);
            } else {
                println!("✅ Dessins sauvegardés dans drawings.json");
            }
        }
        ChartMessage::LoadDrawings => {
            if let Err(e) = app.tools_state.load_from_file("drawings.json") {
                eprintln!("❌ Erreur de chargement: {}", e);
            } else {
                println!("✅ Dessins chargés depuis drawings.json");
            }
        }
        
        // === Position souris ===
        ChartMessage::MouseMoved { position } => {
            app.chart_state.interaction.mouse_position = Some(position);
        }
        
        // === Resize ===
        ChartMessage::Resize { width, height, x, y } => {
            app.chart_state.resize(width, height);
            app.chart_state.interaction.set_main_chart_bounds(x, y, width, height);
        }
        
        // === Backtest ===
        ChartMessage::SelectBacktestDate { time } => {
            // Vérifier si la section Backtest est active
            if app.ui.bottom_panel_sections.active_bottom_section == BottomPanelSection::Backtest {
                // Ne pas permettre de redéfinir la position si la lecture est en cours
                if !app.ui.backtest_state.is_playing {
                    // Mettre à jour le timestamp de départ
                    app.ui.backtest_state.start_timestamp = Some(time);
                    
                    // Réinitialiser les index pour que la barre se positionne sur la nouvelle date
                    app.ui.backtest_state.current_index = 0;
                    app.ui.backtest_state.start_index = None;
                }
            }
        }
    }
}

/// Helper pour finaliser l'édition d'un rectangle avec historique
pub fn finish_rectangle_edit(app: &mut ChartApp) {
    if let (Some(idx), Some(old_rect)) = (
        app.tools_state.editing.selected_index,
        app.tools_state.editing.original_rect.clone(),
    ) {
        if idx < app.tools_state.rectangles.len() {
            let new_rect = app.tools_state.rectangles[idx].clone();
            if old_rect.start_time != new_rect.start_time ||
               old_rect.end_time != new_rect.end_time ||
               old_rect.start_price != new_rect.start_price ||
               old_rect.end_price != new_rect.end_price {
                app.tools_state.history.record(HistoryAction::ModifyRectangle {
                    index: idx,
                    old_rect,
                    new_rect,
                });
            }
        }
    }
    app.tools_state.editing.finish();
}

/// Helper pour finaliser l'édition d'une ligne horizontale avec historique
pub fn finish_hline_edit(app: &mut ChartApp) {
    if let (Some(idx), Some(old_line)) = (
        app.tools_state.hline_editing.selected_index,
        app.tools_state.hline_editing.original_line.clone(),
    ) {
        if idx < app.tools_state.horizontal_lines.len() {
            let new_line = app.tools_state.horizontal_lines[idx].clone();
            if (old_line.price - new_line.price).abs() > 0.0001 {
                app.tools_state.history.record(HistoryAction::ModifyHLine {
                    index: idx,
                    old_line,
                    new_line,
                });
            }
        }
    }
    app.tools_state.hline_editing.finish();
}

/// Helper pour supprimer un élément sélectionné avec historique
pub fn delete_selected(app: &mut ChartApp) {
    // Supprimer rectangle sélectionné
    if let Some(index) = app.tools_state.editing.selected_index {
        if index < app.tools_state.rectangles.len() {
            let deleted_rect = app.tools_state.rectangles[index].clone();
            app.tools_state.history.record(HistoryAction::DeleteRectangle { 
                index, 
                rect: deleted_rect 
            });
            app.tools_state.rectangles.remove(index);
            app.tools_state.editing.deselect();
            return;
        }
    }
    
    // Supprimer ligne horizontale sélectionnée
    if let Some(index) = app.tools_state.hline_editing.selected_index {
        if index < app.tools_state.horizontal_lines.len() {
            let deleted_line = app.tools_state.horizontal_lines[index].clone();
            app.tools_state.history.record(HistoryAction::DeleteHLine { 
                index, 
                line: deleted_line 
            });
            app.tools_state.horizontal_lines.remove(index);
            app.tools_state.hline_editing.deselect();
        }
    }
}
