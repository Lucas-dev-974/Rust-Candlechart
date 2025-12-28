//! Widget Canvas pour afficher le MACD (Moving Average Convergence Divergence)
//!
//! Affiche le MACD dans un graphique séparé sous le graphique principal.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use std::sync::Arc;

use crate::finance_chart::state::ChartState;
use crate::finance_chart::render::{calculate_candle_period, calculate_bar_width};
use crate::finance_chart::render::render_macd_crosshair;
use crate::finance_chart::render::crosshair::CrosshairStyle;
use super::calc::MacdValue;
use super::data::{calculate_macd_data, calculate_macd_range, calculate_all_macd_values};
use super::scaling::MacdScaling;

/// Program Iced pour le rendu du MACD
pub struct MACDProgram<'a> {
    chart_state: &'a ChartState,
    /// Valeurs MACD pré-calculées (optionnel, pour éviter les recalculs)
    precomputed_macd_values: Option<Arc<Vec<Option<MacdValue>>>>,
}

impl<'a> MACDProgram<'a> {
    pub fn new(chart_state: &'a ChartState) -> Self {
        Self { 
            chart_state,
            precomputed_macd_values: None,
        }
    }
    
    /// Crée un nouveau MACDProgram avec des valeurs MACD pré-calculées
    #[allow(dead_code)] // Utilisé pour optimisation future
    pub fn with_precomputed_values(
        chart_state: &'a ChartState, 
        macd_values: Arc<Vec<Option<MacdValue>>>
    ) -> Self {
        Self {
            chart_state,
            precomputed_macd_values: Some(macd_values),
        }
    }
}

impl<'a, Message> Program<Message> for MACDProgram<'a> {
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
        frame.fill(&background, Color::from_rgb(0.08, 0.10, 0.08));

        // Utiliser les valeurs pré-calculées si disponibles, sinon les calculer
        let mut _owned_macd: Option<Vec<Option<MacdValue>>> = None;
        let all_macd_slice: &[Option<MacdValue>] = if let Some(ref precomputed) =
            self.precomputed_macd_values
        {
            &precomputed[..]
        } else {
            let values = match calculate_all_macd_values(self.chart_state) {
                Some(v) => v,
                None => return vec![frame.into_geometry()],
            };
            _owned_macd = Some(values);
            _owned_macd.as_ref().map(|v| &v[..]).unwrap()
        };

        // Extraire les valeurs visibles
        let (visible_macd_values, visible_candles_slice, _visible_start_idx) =
            match calculate_macd_data(self.chart_state, all_macd_slice) {
                Some(data) => data,
                None => return vec![frame.into_geometry()],
            };

        let viewport = &self.chart_state.viewport;
        
        // Créer un TimeScale temporaire pour le MACD chart
        let (min_time, max_time) = viewport.time_scale().time_range();
        use crate::finance_chart::scale::TimeScale;
        let macd_time_scale = TimeScale::new(min_time, max_time, bounds.width);

        let height = bounds.height;

        // Calculer la plage de valeurs MACD pour le scaling
        let (min_macd, max_macd) = match calculate_macd_range(visible_macd_values) {
            Some(range) => range,
            None => return vec![frame.into_geometry()],
        };

        // Créer le scaling MACD
        let scaling = MacdScaling::new(min_macd, max_macd, height);
        let zero_y = scaling.zero_y();
        
        // Calculer les niveaux MACD pour dessiner les lignes de référence
        let macd_step = scaling.calculate_step();
        let first_macd = scaling.first_level();
        
        // Dessiner les lignes de référence horizontales
        let mut macd_value = first_macd;
        while macd_value <= scaling.symmetric_max {
            let y = scaling.macd_to_y(macd_value);
            
            if y >= 0.0 && y <= height {
                let ref_line = Path::new(|builder| {
                    builder.move_to(Point::new(0.0, y));
                    builder.line_to(Point::new(bounds.width, y));
                });
                
                let line_color = if macd_value == 0.0 {
                    Color::from_rgba(0.5, 0.5, 0.5, 0.5)
                } else {
                    Color::from_rgba(0.3, 0.3, 0.3, 0.3)
                };
                
                frame.stroke(
                    &ref_line,
                    Stroke::default()
                        .with_color(line_color)
                        .with_width(1.0),
                );
            }
            
            macd_value += macd_step;
        }

        // Dessiner la ligne MACD
        let macd_path = Path::new(|builder| {
            let mut first_point = true;
            
            for (i, macd_opt) in visible_macd_values.iter().enumerate() {
                if i >= visible_candles_slice.len() {
                    break;
                }
                
                if let Some(macd) = macd_opt {
                    let x = macd_time_scale.time_to_x(visible_candles_slice[i].timestamp);
                    let y = scaling.macd_to_y(macd.macd_line);
                    
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

        frame.stroke(
            &macd_path,
            Stroke::default()
                .with_color(Color::from_rgb(0.0, 0.8, 1.0)) // Cyan
                .with_width(1.5),
        );

        // Dessiner la ligne de signal
        let signal_path = Path::new(|builder| {
            let mut first_point = true;
            
            for (i, macd_opt) in visible_macd_values.iter().enumerate() {
                if i >= visible_candles_slice.len() {
                    break;
                }
                
                if let Some(macd) = macd_opt {
                    let x = macd_time_scale.time_to_x(visible_candles_slice[i].timestamp);
                    let y = scaling.macd_to_y(macd.signal_line);
                    
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

        frame.stroke(
            &signal_path,
            Stroke::default()
                .with_color(Color::from_rgb(1.0, 0.5, 0.0)) // Orange
                .with_width(1.5),
        );

        // Dessiner l'histogramme
        for (i, macd_opt) in visible_macd_values.iter().enumerate() {
            if i >= visible_candles_slice.len() {
                break;
            }
            
            if let Some(macd) = macd_opt {
                let x = macd_time_scale.time_to_x(visible_candles_slice[i].timestamp);
                
                if x >= -10.0 && x <= bounds.width + 10.0 {
                    let histogram_y = scaling.macd_to_y(macd.histogram);
                    let bar_height = (zero_y - histogram_y).abs();
                    let bar_y = if macd.histogram >= 0.0 {
                        zero_y - bar_height
                    } else {
                        zero_y
                    };
                    
                    // Comparer avec la valeur précédente de l'histogramme
                    let is_histogram_decreasing = if i > 0 {
                        if let Some(prev_macd) = visible_macd_values.get(i - 1).and_then(|opt| opt.as_ref()) {
                            macd.histogram.abs() < prev_macd.histogram.abs()
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    
                    let bar_color = if macd.histogram >= 0.0 {
                        if is_histogram_decreasing {
                            Color::from_rgba(0.3, 1.0, 0.3, 0.4)
                        } else {
                            Color::from_rgba(0.0, 0.8, 0.0, 0.6)
                        }
                    } else {
                        if is_histogram_decreasing {
                            Color::from_rgba(1.0, 0.3, 0.3, 0.4)
                        } else {
                            Color::from_rgba(0.8, 0.0, 0.0, 0.6)
                        }
                    };
                    
                    // Calculer la largeur des barres
                    let candle_period = calculate_candle_period(visible_candles_slice);
                    let bar_width = calculate_bar_width(
                        candle_period,
                        max_time - min_time,
                        bounds.width,
                    );
                    
                    let bar = Path::rectangle(
                        Point::new(x - bar_width / 2.0, bar_y),
                        iced::Size::new(bar_width.max(1.0), bar_height.max(1.0)),
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
        
        render_macd_crosshair(
            &mut frame,
            viewport,
            main_mouse_position,
            bounds.width,
            bounds.height,
            mouse_position_in_chart.map(|p| p.y),
            &|y| scaling.y_to_macd(y),
            Some(crosshair_style),
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
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
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::default()
    }
}

/// Crée un widget canvas pour le MACD
pub fn macd_chart<'a>(chart_state: &'a ChartState) -> Element<'a, crate::app::messages::Message> {
    Canvas::new(MACDProgram::new(chart_state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

