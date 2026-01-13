//! Composant overlay pour la barre verticale de sélection de date du backtest

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Element, Event, Length, Point, Rectangle, mouse, Color};
use iced::mouse::Cursor;
use crate::finance_chart::state::ChartState;
use crate::finance_chart::X_AXIS_HEIGHT;
use crate::app::state::backtest::BacktestState;

/// État local du canvas pour le drag (UI uniquement)
#[derive(Debug, Clone, Default)]
pub struct BacktestOverlayState {
    /// Position X locale pendant le drag (pour affichage immédiat)
    drag_x: Option<f32>,
}

/// Programme canvas pour la barre verticale de sélection du backtest
pub struct BacktestOverlayProgram<'a> {
    chart_state: &'a ChartState,
    backtest_state: &'a BacktestState,
}

impl<'a> BacktestOverlayProgram<'a> {
    pub fn new(chart_state: &'a ChartState, backtest_state: &'a BacktestState) -> Self {
        Self { chart_state, backtest_state }
    }
}

impl<'a> Program<crate::app::messages::Message> for BacktestOverlayProgram<'a> {
    type State = BacktestOverlayState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner la barre verticale seulement si le backtest est activé
        if !self.backtest_state.enabled {
            return vec![frame.into_geometry()];
        }
        
        // Pendant le drag, utiliser la position locale pour un suivi fluide
        // Sinon, utiliser le timestamp de l'état global
        let x = if let Some(drag_x) = state.drag_x {
            drag_x
        } else {
            let timestamp = self.backtest_state.current_candle_timestamp();
            if let Some(timestamp) = timestamp {
                let viewport = &self.chart_state.viewport;
                viewport.time_scale().time_to_x(timestamp)
            } else {
                return vec![frame.into_geometry()];
            }
        };
        
        let viewport = &self.chart_state.viewport;
        let chart_width = viewport.width();
        
        if chart_width > 0.0 && x >= 0.0 && x <= chart_width {
            // Dessiner la ligne verticale sur toute la hauteur (sauf l'axe X en bas)
            let chart_height = bounds.height - X_AXIS_HEIGHT;
            
            if chart_height > 0.0 {
                // Couleur différente pour la barre de sélection du backtest (orange/rouge)
                let line_color = if self.backtest_state.is_playing {
                    Color::from_rgb(0.2, 0.8, 0.3) // Vert quand en lecture
                } else {
                    Color::from_rgb(0.9, 0.5, 0.1) // Orange quand sélectionné mais pas en lecture
                };
                
                let vertical_line = Path::new(|builder| {
                    builder.move_to(Point::new(x, 0.0));
                    builder.line_to(Point::new(x, chart_height));
                });
                
                let stroke = Stroke::default()
                    .with_color(line_color)
                    .with_width(2.0); // Plus épais que le crosshair
                
                frame.stroke(&vertical_line, stroke);
            }
        }
        
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<crate::app::messages::Message>> {
        use iced::widget::canvas::Action;
        
        // Ne gérer les événements que si le backtest est activé et pas en lecture
        if !self.backtest_state.enabled || self.backtest_state.is_playing {
            // Réinitialiser l'état local si le backtest est désactivé
            state.drag_x = None;
            return None;
        }
        
        // Récupérer le timestamp actuel pour calculer la position X de la ligne
        let timestamp = self.backtest_state.current_candle_timestamp();
        
        let Some(timestamp) = timestamp else {
            state.drag_x = None;
            return None;
        };
        
        let viewport = &self.chart_state.viewport;
        let chart_width = viewport.width();
        if chart_width <= 0.0 {
            state.drag_x = None;
            return None;
        }
        
        let playhead_x = viewport.time_scale().time_to_x(timestamp);
        
        // Zone de détection autour de la ligne (5 pixels de chaque côté)
        const HIT_ZONE: f32 = 5.0;
        
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    // Vérifier si le clic est proche de la ligne
                    let distance = (position.x - playhead_x).abs();
                    if distance <= HIT_ZONE && position.x >= 0.0 && position.x <= chart_width {
                        // Démarrer le drag
                        if let Some(global_pos) = cursor.position() {
                            // Initialiser la position locale
                            state.drag_x = Some(position.x);
                            return Some(Action::publish(crate::app::messages::Message::StartDragPlayhead(global_pos)));
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position: _ }) => {
                if self.backtest_state.dragging_playhead {
                    // Mettre à jour la position locale pour un suivi fluide
                    if let Some(position) = cursor.position_in(bounds) {
                        // Mettre à jour la position locale immédiatement pour que la ligne suive le curseur
                        state.drag_x = Some(position.x);
                        
                        // Forcer un redraw immédiat avec la position locale
                        // Le canvas se redessinera immédiatement avec state.drag_x
                        // On publie aussi le message pour mettre à jour l'état global
                        if let Some(global_pos) = cursor.position() {
                            // Publier le message pour mettre à jour l'état global
                            // Le canvas se redessinera immédiatement grâce au changement de state.drag_x
                            // et au redraw déclenché par render_version dans le handler
                            return Some(Action::publish(crate::app::messages::Message::UpdateDragPlayhead(global_pos)));
                        }
                        // Si pas de position globale, juste demander un redraw avec la position locale
                        return Some(Action::request_redraw());
                    }
                } else {
                    // Si on n'est plus en train de drag, réinitialiser la position locale
                    state.drag_x = None;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.backtest_state.dragging_playhead {
                    // Terminer le drag et réinitialiser la position locale
                    state.drag_x = None;
                    return Some(Action::publish(crate::app::messages::Message::EndDragPlayhead));
                }
            }
            _ => {}
        }
        
        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        
        // Ne gérer que si le backtest est activé et pas en lecture
        if !self.backtest_state.enabled || self.backtest_state.is_playing {
            return mouse::Interaction::default();
        }
        
        // Vérifier si le curseur est proche de la ligne
        let timestamp = self.backtest_state.current_candle_timestamp();
        
        if let Some(timestamp) = timestamp {
            if let Some(position) = cursor.position_in(bounds) {
                let viewport = &self.chart_state.viewport;
                let chart_width = viewport.width();
                if chart_width > 0.0 {
                    let playhead_x = viewport.time_scale().time_to_x(timestamp);
                    const HIT_ZONE: f32 = 5.0;
                    let distance = (position.x - playhead_x).abs();
                    if distance <= HIT_ZONE && position.x >= 0.0 && position.x <= chart_width {
                        return mouse::Interaction::Grab;
                    }
                }
            }
        }
        
        mouse::Interaction::default()
    }
}

/// Crée un canvas overlay pour la barre verticale de sélection du backtest
pub fn backtest_overlay<'a>(
    chart_state: &'a ChartState,
    backtest_state: &'a BacktestState,
) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(BacktestOverlayProgram::new(chart_state, backtest_state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}


