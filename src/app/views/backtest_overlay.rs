//! Composant overlay pour la barre verticale de sélection de date du backtest

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Element, Event, Length, Point, Rectangle, mouse, Color};
use iced::mouse::Cursor;
use crate::finance_chart::state::ChartState;
use crate::finance_chart::X_AXIS_HEIGHT;
use crate::app::state::backtest::BacktestState;

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

impl<'a, Message> Program<Message> for BacktestOverlayProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Dessiner la barre verticale si une date est sélectionnée
        // Utiliser le timestamp actuel de la bougie (même si le player est arrêté)
        let timestamp = if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
            let all_candles = active_series.data.all_candles();
            // Utiliser current_candle_timestamp si on a un start_index (backtest démarré),
            // sinon utiliser start_timestamp (pas encore démarré)
            if self.backtest_state.start_index.is_some() {
                self.backtest_state.current_candle_timestamp(all_candles)
            } else {
                self.backtest_state.start_timestamp
            }
        } else {
            self.backtest_state.start_timestamp
        };
        
        if let Some(timestamp) = timestamp {
            let viewport = &self.chart_state.viewport;
            let chart_width = viewport.width();
            
            if chart_width > 0.0 {
                let x = viewport.time_scale().time_to_x(timestamp);
                
                if x >= 0.0 && x <= chart_width {
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
            }
        }
        
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> mouse::Interaction {
        // Permettre aux événements de passer à travers (pas de blocage)
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


