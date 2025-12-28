//! Canvas pour l'axe Y du MACD
//! 
//! Affiche les valeurs du MACD sur l'axe vertical à droite du graphique MACD.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text, Path, Stroke};
use iced::widget::stack;
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use iced::Pixels;

use super::axis_canvas::Y_AXIS_WIDTH;
use super::axis_style::AxisStyle;
use super::state::ChartState;
use super::indicators::MacdValue;
use super::macd_data::{calculate_macd_data, calculate_macd_range, get_last_macd_value};
use super::macd_scaling::MacdScaling;
use std::sync::Arc;

/// Program pour l'axe Y du MACD
pub struct MACDAxisProgram {
    chart_state: ChartState,
    /// Valeurs MACD pré-calculées (optionnel, pour éviter les recalculs)
    precomputed_macd_values: Option<Arc<Vec<Option<MacdValue>>>>,
}

impl MACDAxisProgram {
    pub fn new(chart_state: ChartState) -> Self {
        Self { 
            chart_state,
            precomputed_macd_values: None,
        }
    }
    
    /// Crée un nouveau MACDAxisProgram avec des valeurs MACD pré-calculées
    pub fn with_precomputed_values(chart_state: ChartState, macd_values: Arc<Vec<Option<MacdValue>>>) -> Self {
        Self {
            chart_state,
            precomputed_macd_values: Some(macd_values),
        }
    }
}

impl<Message> Program<Message> for MACDAxisProgram {
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
        let style = AxisStyle::default();

        // Fond
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, style.background_color);

        // Utiliser les valeurs pré-calculées si disponibles, sinon les calculer.
        // Travailler sur des slices pour éviter des clones coûteux.
        let mut _owned_macd: Option<Vec<Option<MacdValue>>> = None;
        let all_macd_slice: &[Option<MacdValue>] = if let Some(ref precomputed) = self.precomputed_macd_values {
            &precomputed[..]
        } else {
            let values = match super::macd_data::calculate_all_macd_values(&self.chart_state) {
                Some(v) => v,
                None => return vec![frame.into_geometry()],
            };
            _owned_macd = Some(values);
            _owned_macd.as_ref().map(|v| &v[..]).unwrap()
        };

        // Extraire les valeurs visibles (les références pointent vers all_macd_slice)
        let (visible_macd_values, _visible_candles_slice, _visible_start_idx) =
            match calculate_macd_data(&self.chart_state, all_macd_slice) {
                Some(data) => data,
                None => return vec![frame.into_geometry()],
            };

        // Calculer la plage de valeurs MACD pour le scaling
        let (min_macd, max_macd) = match calculate_macd_range(&visible_macd_values) {
            Some(range) => range,
            None => return vec![frame.into_geometry()],
        };

        // Créer le scaling MACD
        let scaling = MacdScaling::new(min_macd, max_macd, bounds.height);

        // La position Y de zéro est toujours au centre
        let zero_y = scaling.zero_y();
        if zero_y >= 0.0 && zero_y <= bounds.height {
            let zero_line = Path::new(|builder| {
                builder.move_to(Point::new(0.0, zero_y));
                builder.line_to(Point::new(bounds.width, zero_y));
            });
            frame.stroke(
                &zero_line,
                Stroke::default()
                    .with_color(Color::from_rgba(0.5, 0.5, 0.5, 0.5))
                    .with_width(1.0),
            );
        }

        // Calculer les niveaux MACD à afficher
        let macd_step = scaling.calculate_step();
        let first_macd = scaling.first_level();

        // Dessiner les labels Y
        let mut macd_value = first_macd;
        while macd_value <= scaling.symmetric_max {
            let y = scaling.macd_to_y(macd_value);
            
            // Ne dessiner que si visible
            if y >= 0.0 && y <= bounds.height {
                // Formater le MACD selon la précision nécessaire
                let label = if macd_step >= 1.0 {
                    format!("{:.0}", macd_value)
                } else if macd_step >= 0.1 {
                    format!("{:.1}", macd_value)
                } else if macd_step >= 0.01 {
                    format!("{:.2}", macd_value)
                } else {
                    format!("{:.4}", macd_value)
                };

                let text = Text {
                    content: label,
                    position: Point::new(bounds.width - 5.0 - 15.0, y - 6.0), // Décaler de 15px vers la gauche
                    color: style.text_color,
                    size: Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }

            macd_value += macd_step;
        }

        vec![frame.into_geometry()]
    }
}

/// Program pour l'overlay du label MACD qui suit la valeur
pub struct MACDLabelOverlayProgram {
    macd_value: Option<MacdValue>,
}

impl MACDLabelOverlayProgram {
    pub fn new(macd_value: Option<MacdValue>) -> Self {
        Self { macd_value }
    }
}

impl<Message> Program<Message> for MACDLabelOverlayProgram {
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

        if let Some(macd) = &self.macd_value {
            // Pour le MACD, on affiche la valeur MACD line
            // La position Y sera calculée dynamiquement en fonction de la plage de valeurs
            // Pour simplifier, on affiche juste le label en haut à gauche
            let text = Text {
                content: format!("MACD: {:.4}", macd.macd_line),
                position: Point::new(5.0, 10.0),
                color: Color::from_rgb(0.0, 0.8, 1.0), // Cyan pour correspondre à la ligne MACD
                size: Pixels(11.0),
                ..Text::default()
            };
            frame.fill_text(text);
        }

        vec![frame.into_geometry()]
    }
}

/// Crée un widget canvas pour l'axe Y du MACD avec overlay de la valeur MACD actuelle
pub fn macd_y_axis<'a>(chart_state: &'a ChartState) -> Element<'a, crate::app::messages::Message> {
    // Calculer toutes les valeurs MACD une seule fois (réutilisées dans le draw et l'overlay)
    let all_vec = match super::macd_data::calculate_all_macd_values(chart_state) {
        Some(v) => v,
        None => {
            // Si pas de valeurs MACD, créer un canvas vide
            return Canvas::new(MACDAxisProgram::new(chart_state.clone()))
                .width(Length::Fixed(Y_AXIS_WIDTH))
                .height(Length::Fill)
                .into();
        }
    };

    // Placer les valeurs dans un Arc pour éviter les clones larges
    let arc = Arc::new(all_vec);

    // Récupérer la dernière valeur MACD en utilisant les valeurs pré-calculées
    let current_macd = get_last_macd_value(chart_state, Some(&*arc));

    // Créer l'axe Y avec les valeurs MACD pré-calculées pour éviter le recalcul
    let axis = Canvas::new(MACDAxisProgram::with_precomputed_values(
        chart_state.clone(),
        arc.clone()
    ))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    // Créer l'overlay avec le label MACD
    let overlay = Canvas::new(MACDLabelOverlayProgram::new(current_macd))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    // Stacker l'overlay sur l'axe Y
    stack![
        axis,
        overlay
    ]
    .width(Length::Fixed(Y_AXIS_WIDTH))
    .height(Length::Fill)
    .into()
}

