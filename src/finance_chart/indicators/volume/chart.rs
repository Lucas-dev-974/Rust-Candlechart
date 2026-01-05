//! Widget Canvas pour afficher les volumes échangés
//!
//! Affiche des barres de volume pour chaque bougie, synchronisé avec le graphique principal.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced::mouse::Cursor;

use crate::finance_chart::state::ChartState;
use crate::finance_chart::scale::VolumeScale;
use crate::finance_chart::render::{calculate_bar_width, calculate_candle_period, render_volume_crosshair};
use crate::finance_chart::render::crosshair::CrosshairStyle;

/// Program Iced pour le rendu du volume
pub struct VolumeProgram<'a> {
    chart_state: &'a ChartState,
    volume_scale: VolumeScale,
}

impl<'a> VolumeProgram<'a> {
    pub fn new(chart_state: &'a ChartState, volume_scale: VolumeScale) -> Self {
        Self {
            chart_state,
            volume_scale,
        }
    }
}

impl<'a> Program<crate::app::messages::Message> for VolumeProgram<'a> {
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

        // Fond sombre
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, Color::from_rgb(0.08, 0.08, 0.10));

        // Récupérer les bougies visibles
        let visible_candles = self.chart_state.visible_candles();
        
        if visible_candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        // Utiliser la première série active
        let (_, candles) = &visible_candles[0];
        let viewport = &self.chart_state.viewport;
        
        // Créer un TimeScale temporaire pour le volume chart
        let (min_time, max_time) = viewport.time_scale().time_range();
        use crate::finance_chart::scale::TimeScale;
        let volume_time_scale = TimeScale::new(min_time, max_time, bounds.width);

        // Calculer la largeur des barres
        let candle_period = calculate_candle_period(candles);
        let bar_width = calculate_bar_width(candle_period, max_time - min_time, bounds.width);

        // Dessiner les barres de volume
        for candle in candles.iter() {
            if candle.volume.is_nan() || candle.volume < 0.0 {
                continue;
            }
            
            let x = volume_time_scale.time_to_x(candle.timestamp);
            
            if x >= -bar_width * 3.0 && x <= bounds.width + bar_width * 3.0 {
                let y_bottom = bounds.height;
                let y_top = self.volume_scale.volume_to_y(candle.volume);
                
                let bar_height = (y_bottom - y_top).max(0.0);

                if bar_height >= 0.0 {
                    let bar_color = if candle.is_bullish() {
                        Color::from_rgba(0.0, 0.6, 0.0, 0.7) // Vert
                    } else {
                        Color::from_rgba(0.8, 0.0, 0.0, 0.7) // Rouge
                    };

                    let final_height = bar_height.max(1.0);
                    let final_y_top = y_bottom - final_height;

                    let bar = Path::rectangle(
                        Point::new(x - bar_width / 2.0, final_y_top),
                        Size::new(bar_width, final_height),
                    );
                    frame.fill(&bar, bar_color);
                }
            }
        }

        // Rendre le crosshair synchronisé avec le graphique principal
        let mouse_position_in_chart = cursor.position_in(bounds);
        let main_mouse_position = self.chart_state.interaction.mouse_position;
        
        let crosshair_style = CrosshairStyle {
            line_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            ..Default::default()
        };
        
        render_volume_crosshair(
            &mut frame,
            viewport,
            main_mouse_position,
            &self.volume_scale,
            bounds.width,
            bounds.height,
            mouse_position_in_chart.map(|p| p.y),
            Some(crosshair_style),
            mouse_position_in_chart.map(|p| p.x),
        );

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
                                    position: absolute_position,
                                    time: None,
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

/// Crée un élément canvas pour afficher les volumes
pub fn volume_chart<'a>(
    chart_state: &'a ChartState,
    volume_scale: VolumeScale,
) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(VolumeProgram::new(chart_state, volume_scale))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

