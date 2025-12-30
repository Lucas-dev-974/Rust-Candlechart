//! Composant overlay pour la barre verticale synchronisée du crosshair
//!
//! Ce composant dessine une ligne verticale qui traverse tous les graphiques
//! (principal, volume, RSI, MACD) de manière synchronisée.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Element, Event, Length, Point, Rectangle, mouse};
use iced::mouse::Cursor;
use crate::finance_chart::state::ChartState;
use crate::finance_chart::render::crosshair::{CrosshairStyle, draw_time_label};
use crate::finance_chart::render::grid::format_time;
use crate::finance_chart::scale::TimeScale;
use crate::finance_chart::X_AXIS_HEIGHT;

/// Programme canvas pour le crosshair vertical synchronisé
pub struct CrosshairOverlayProgram<'a> {
    chart_state: &'a ChartState,
}

impl<'a> CrosshairOverlayProgram<'a> {
    pub fn new(chart_state: &'a ChartState) -> Self {
        Self { chart_state }
    }
}

impl<'a, Message> Program<Message> for CrosshairOverlayProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        
        // Utiliser la position de la souris du graphique principal ou la position actuelle du curseur
        let timestamp = if let Some(pos) = cursor.position() {
            // Convertir en position relative dans les bounds
            let relative_pos = Point::new(pos.x - bounds.x, pos.y - bounds.y);
            
            // Vérifier si la souris est dans la zone des graphiques (pas dans l'axe Y)
            // L'axe Y fait environ 60px de large
            let chart_width = bounds.width - 60.0;
            
            if relative_pos.x >= 0.0 && relative_pos.x <= chart_width && chart_width > 0.0 {
                // Utiliser le viewport principal pour calculer le timestamp
                let viewport = &self.chart_state.viewport;
                let (min_time, max_time) = viewport.time_scale().time_range();
                let time_scale = TimeScale::new(min_time, max_time, chart_width);
                Some(time_scale.x_to_time(relative_pos.x))
            } else {
                None
            }
        } else {
            None
        };
        
        // Si on n'a pas de timestamp depuis le curseur, utiliser la position du graphique principal
        let timestamp = timestamp.or_else(|| {
            if let Some(main_pos) = self.chart_state.interaction.mouse_position {
                let viewport = &self.chart_state.viewport;
                Some(viewport.time_scale().x_to_time(main_pos.x))
            } else {
                None
            }
        });
        
        // Dessiner la ligne verticale si on a un timestamp
        if let Some(timestamp) = timestamp {
            let viewport = &self.chart_state.viewport;
            let (min_time, max_time) = viewport.time_scale().time_range();
            let chart_width = bounds.width - 60.0;
            
            if chart_width > 0.0 {
                let time_scale = TimeScale::new(min_time, max_time, chart_width);
                let x = time_scale.time_to_x(timestamp);
                
                if x >= 0.0 && x <= chart_width {
                    let style = CrosshairStyle::default();
                    
                    // Dessiner la ligne verticale sur toute la hauteur (sauf l'axe X en bas)
                    let chart_height = bounds.height - X_AXIS_HEIGHT;
                    
                    if chart_height > 0.0 {
                        let vertical_line = Path::new(|builder| {
                            builder.move_to(Point::new(x, 0.0));
                            builder.line_to(Point::new(x, chart_height));
                        });
                        
                        let stroke = Stroke::default()
                            .with_color(style.line_color)
                            .with_width(style.line_width);
                        
                        frame.stroke(&vertical_line, stroke);
                        
                        // Dessiner le label du temps sur le bord bas (dans l'axe X)
                        let time_label = format_time(timestamp, 3600);
                        draw_time_label(&mut frame, &style, x, bounds.height, &time_label);
                    }
                }
            }
        }
        
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        // Demander un redraw lors des mouvements de souris
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                Some(iced::widget::canvas::Action::request_redraw())
            }
            _ => None,
        }
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

/// Crée un canvas overlay pour le crosshair vertical synchronisé
pub fn crosshair_overlay<'a>(
    chart_state: &'a ChartState,
) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(CrosshairOverlayProgram::new(chart_state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

