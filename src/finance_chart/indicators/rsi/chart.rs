//! Widget Canvas pour afficher le RSI (Relative Strength Index)
//!
//! Affiche le RSI dans un graphique séparé sous le graphique principal.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;

use crate::finance_chart::state::ChartState;
use crate::finance_chart::render::render_rsi_crosshair;
use crate::finance_chart::render::crosshair::CrosshairStyle;
use super::calc::{RSI_OVERBOUGHT, RSI_OVERSOLD};
use super::data::{calculate_all_rsi_values, calculate_rsi_data, get_last_rsi_value};
use iced::widget::canvas::Text;

/// Program Iced pour le rendu du RSI
pub struct RSIProgram<'a> {
    chart_state: &'a ChartState,
}

impl<'a> RSIProgram<'a> {
    pub fn new(chart_state: &'a ChartState) -> Self {
        Self { chart_state }
    }
}

impl<'a> Program<crate::app::messages::Message> for RSIProgram<'a> {
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

        // Fond sombre (légèrement différent du volume pour les distinguer visuellement)
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::from_rgb(0.10, 0.08, 0.10)); // Légèrement plus rouge que le volume

        // Calculer toutes les valeurs RSI d'abord (sur toutes les bougies, pas seulement visibles)
        let all_rsi_values = match calculate_all_rsi_values(self.chart_state) {
            Some(values) => values,
            None => return vec![frame.into_geometry()],
        };
        
        // Extraire les valeurs visibles (les références pointent vers all_rsi_values)
        let (visible_rsi_values, visible_candles_slice, _visible_start_idx) =
            match calculate_rsi_data(self.chart_state, &all_rsi_values) {
                Some(data) => data,
                None => return vec![frame.into_geometry()],
            };
        
        if visible_rsi_values.is_empty() || visible_candles_slice.is_empty() {
            return vec![frame.into_geometry()];
        }

        let viewport = &self.chart_state.viewport;
        
        // Créer un TimeScale temporaire pour le RSI chart qui utilise bounds.width
        let (min_time, max_time) = viewport.time_scale().time_range();
        use crate::finance_chart::scale::TimeScale;
        let rsi_time_scale = TimeScale::new(min_time, max_time, bounds.width);

        let height = bounds.height;

        // Dessiner les zones de sur-achat et sur-vente
        // Zone de sur-achat (70-100)
        let overbought_path = Path::new(|builder| {
            let x1 = 0.0;
            let x2 = bounds.width;
            let y1 = height * (1.0 - RSI_OVERBOUGHT / 100.0) as f32;
            let y2 = 0.0;
            
            builder.move_to(Point::new(x1, y1));
            builder.line_to(Point::new(x2, y1));
            builder.line_to(Point::new(x2, y2));
            builder.line_to(Point::new(x1, y2));
            builder.close();
        });
        frame.fill(&overbought_path, Color::from_rgba(1.0, 0.0, 0.0, 0.2));

        // Zone de sur-vente (0-30)
        let oversold_path = Path::new(|builder| {
            let x1 = 0.0;
            let x2 = bounds.width;
            let y1 = height;
            let y2 = height * (1.0 - RSI_OVERSOLD / 100.0) as f32;
            
            builder.move_to(Point::new(x1, y1));
            builder.line_to(Point::new(x2, y1));
            builder.line_to(Point::new(x2, y2));
            builder.line_to(Point::new(x1, y2));
            builder.close();
        });
        frame.fill(&oversold_path, Color::from_rgba(0.0, 1.0, 0.0, 0.2));

        // Dessiner les lignes de référence à 30, 50 et 70
        let reference_levels = [30.0, 50.0, 70.0];
        for level in reference_levels {
            let y = height * (1.0 - level / 100.0);
            let ref_path = Path::new(|builder| {
                builder.move_to(Point::new(0.0, y));
                builder.line_to(Point::new(bounds.width, y));
            });
            
            let ref_stroke = Stroke::default()
                .with_color(Color::from_rgba(0.5, 0.5, 0.5, 0.5))
                .with_width(1.0);
            frame.stroke(&ref_path, ref_stroke);
        }

        // Dessiner la ligne du RSI
        let rsi_path = Path::new(|builder| {
            let mut first_point = true;
            
            for (i, rsi_opt) in visible_rsi_values.iter().enumerate() {
                if i >= visible_candles_slice.len() {
                    break;
                }
                
                if let Some(rsi) = rsi_opt {
                    let x = rsi_time_scale.time_to_x(visible_candles_slice[i].timestamp);
                    let normalized_rsi = (*rsi / 100.0).clamp(0.0, 1.0);
                    let y = height * (1.0 - normalized_rsi as f32);
                    
                    if x >= -10.0 && x <= bounds.width + 10.0 {
                        if first_point {
                            builder.move_to(Point::new(x, y));
                            first_point = false;
                        } else {
                            builder.line_to(Point::new(x, y));
                        }
                    }
                }
            }
        });

        let stroke = Stroke::default()
            .with_color(Color::from_rgb(0.0, 0.8, 1.0)) // Cyan
            .with_width(2.0);
        frame.stroke(&rsi_path, stroke);

        // Rendre le crosshair synchronisé avec le graphique principal
        let mouse_position_in_chart = cursor.position_in(bounds);
        let main_mouse_position = self.chart_state.interaction.mouse_position;
        
        let crosshair_style = CrosshairStyle {
            line_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            ..Default::default()
        };
        
        render_rsi_crosshair(
            &mut frame,
            viewport,
            main_mouse_position,
            bounds.width,
            bounds.height,
            mouse_position_in_chart.map(|p| p.y),
            Some(crosshair_style),
            mouse_position_in_chart.map(|p| p.x),
        );

        // Dessiner le label RSI dans la zone du chart (à droite), afin qu'il ne soit pas tronqué
        if let Some(current_rsi) = get_last_rsi_value(self.chart_state, Some(&all_rsi_values)) {
            let label = format!("RSI: {:.1}", current_rsi);
            let text = Text {
                content: label,
                position: Point::new(bounds.width - 70.0, 6.0),
                color: Color::from_rgb(0.0, 0.8, 1.0),
                size: iced::Pixels(11.0),
                ..Text::default()
            };
            // Background rect for contrast
            let text_bg = Path::rectangle(Point::new(bounds.width - 74.0, 0.0), iced::Size::new(74.0, 18.0));
            frame.fill(&text_bg, Color::from_rgba(0.0, 0.0, 0.0, 0.45));
            frame.fill_text(text);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<iced::widget::canvas::Action<crate::app::messages::Message>> {
        match event {
            // Gestion du pan (drag)
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                // Utiliser la position absolue du curseur pour cohérence avec le graphique principal
                if let Some(_) = cursor.position_in(bounds) {
                    // Utiliser la position absolue (par rapport à la fenêtre) pour le pan
                    // Le graphique principal convertira cette position en position relative à ses bounds
                    if let Some(absolute_position) = cursor.position() {
                        return Some(iced::widget::canvas::Action::publish(
                            crate::app::messages::Message::Chart(
                                crate::finance_chart::messages::ChartMessage::StartPan { 
                                    position: absolute_position 
                                }
                            )
                        ));
                    }
                }
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                return Some(iced::widget::canvas::Action::publish(
                    crate::app::messages::Message::Chart(
                        crate::finance_chart::messages::ChartMessage::EndPan
                    )
                ));
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: _ }) => {
                // Si on est en train de faire un pan, mettre à jour uniquement l'axe horizontal
                if self.chart_state.interaction.is_panning {
                    // Utiliser la position absolue du curseur pour cohérence avec le graphique principal
                    if let Some(absolute_position) = cursor.position() {
                        return Some(iced::widget::canvas::Action::publish(
                            crate::app::messages::Message::Chart(
                                crate::finance_chart::messages::ChartMessage::UpdatePanHorizontal { 
                                    position: absolute_position 
                                }
                            )
                        ));
                    }
                }
                // Sinon, demander un redraw pour mettre à jour le crosshair
                return Some(iced::widget::canvas::Action::request_redraw());
            }
            _ => {}
        }
        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> iced::mouse::Interaction {
        // Afficher une croix comme curseur
        iced::mouse::Interaction::Crosshair
    }
}

/// Crée un widget canvas pour le RSI
pub fn rsi_chart<'a>(chart_state: &'a ChartState) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(RSIProgram::new(chart_state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

