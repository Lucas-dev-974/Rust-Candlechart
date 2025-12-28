//! Canvas pour l'axe Y du MACD
//! 
//! Affiche les valeurs du MACD sur l'axe vertical à droite du graphique MACD.
//! Utilise un snapshot léger pour éviter de cloner le ChartState complet.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text, Path, Stroke};
use iced::widget::stack;
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use iced::Pixels;

use crate::finance_chart::axis::{Y_AXIS_WIDTH, AxisStyle};
use crate::finance_chart::state::ChartState;
use super::calc::MacdValue;
use super::data::{calculate_macd_data, calculate_macd_range, get_last_macd_value, calculate_all_macd_values};
use super::snapshot::MacdAxisSnapshot;
use super::scaling::MacdScaling;

/// Program pour l'axe Y du MACD
pub struct MACDAxisProgram {
    snapshot: MacdAxisSnapshot,
}

impl MACDAxisProgram {
    pub fn new(snapshot: MacdAxisSnapshot) -> Self {
        Self { snapshot }
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

        // Vérifier que le snapshot contient des données valides
        if !self.snapshot.is_valid() {
            return vec![frame.into_geometry()];
        }

        // Créer le scaling MACD à partir des données pré-calculées
        let scaling = MacdScaling::new(self.snapshot.min_macd, self.snapshot.max_macd, bounds.height);

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
                    position: Point::new(bounds.width - 5.0 - 15.0, y - 6.0),
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
            let text = Text {
                content: format!("MACD: {:.4}", macd.macd_line),
                position: Point::new(5.0, 10.0),
                color: Color::from_rgb(0.0, 0.8, 1.0), // Cyan
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
    // Calculer toutes les valeurs MACD une seule fois
    let all_macd_values = match calculate_all_macd_values(chart_state) {
        Some(v) => v,
        None => {
            let empty_snapshot = MacdAxisSnapshot::new(vec![], 0.0, 0.0);
            return Canvas::new(MACDAxisProgram::new(empty_snapshot))
                .width(Length::Fixed(Y_AXIS_WIDTH))
                .height(Length::Fill)
                .into();
        }
    };

    // Extraire les valeurs visibles
    let (visible_macd_slice, _, _) = match calculate_macd_data(chart_state, &all_macd_values) {
        Some(data) => data,
        None => {
            let empty_snapshot = MacdAxisSnapshot::new(vec![], 0.0, 0.0);
            return Canvas::new(MACDAxisProgram::new(empty_snapshot))
                .width(Length::Fixed(Y_AXIS_WIDTH))
                .height(Length::Fill)
                .into();
        }
    };

    // Calculer la plage de valeurs
    let (min_macd, max_macd) = match calculate_macd_range(visible_macd_slice) {
        Some(range) => range,
        None => {
            let empty_snapshot = MacdAxisSnapshot::new(vec![], 0.0, 0.0);
            return Canvas::new(MACDAxisProgram::new(empty_snapshot))
                .width(Length::Fixed(Y_AXIS_WIDTH))
                .height(Length::Fill)
                .into();
        }
    };

    // Récupérer la dernière valeur MACD pour l'overlay
    let last_macd = get_last_macd_value(chart_state, Some(&all_macd_values));

    // Créer le snapshot léger avec les données pré-calculées
    let snapshot = MacdAxisSnapshot::new(
        visible_macd_slice.to_vec(),
        min_macd,
        max_macd,
    );

    // Créer l'axe Y avec le snapshot
    let axis = Canvas::new(MACDAxisProgram::new(snapshot))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    // Créer l'overlay avec le label MACD
    let overlay = Canvas::new(MACDLabelOverlayProgram::new(last_macd))
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

