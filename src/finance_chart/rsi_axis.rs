//! Canvas pour l'axe Y du RSI
//! 
//! Affiche les valeurs du RSI (0-100) sur l'axe vertical à droite du graphique RSI.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Text, Path, Stroke};
use iced::widget::stack;
use iced::{Color, Element, Length, Point, Rectangle};
use iced::mouse::Cursor;
use iced::Pixels;

use super::axis_canvas::Y_AXIS_WIDTH;
use super::axis_style::AxisStyle;
use super::state::ChartState;
use super::rsi_data::{calculate_all_rsi_values, get_last_rsi_value};

/// Program pour l'axe Y du RSI
pub struct RSIAxisProgram {
    height: f32,
}

impl RSIAxisProgram {
    pub fn new(height: f32) -> Self {
        Self { height }
    }
}

/// Program pour l'overlay du label RSI qui suit la valeur
pub struct RSILabelOverlayProgram {
    height: f32,
    rsi_value: Option<f64>,
}

impl RSILabelOverlayProgram {
    pub fn new(height: f32, rsi_value: Option<f64>) -> Self {
        Self { height, rsi_value }
    }
}

impl<Message> Program<Message> for RSILabelOverlayProgram {
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

        if let Some(rsi_value) = self.rsi_value {
            // Mapper le RSI (0-100) sur la hauteur du canvas
            // RSI 100 = haut (y = 0), RSI 0 = bas (y = height)
            let normalized_rsi = (rsi_value / 100.0).clamp(0.0, 1.0);
            let y = self.height * (1.0 - normalized_rsi as f32);
            
            // S'assurer que le label est dans les bounds
            if y >= 0.0 && y <= bounds.height {
                // Dessiner une ligne horizontale pour indiquer la position
                let line_path = Path::new(|builder| {
                    builder.move_to(Point::new(0.0, y));
                    builder.line_to(Point::new(bounds.width, y));
                });
                frame.stroke(
                    &line_path,
                    Stroke::default()
                        .with_color(Color::from_rgba(0.0, 0.8, 1.0, 0.3)) // Cyan transparent
                        .with_width(1.0),
                );
                
                // Dessiner le label à gauche de l'axe
                let text = Text {
                    content: format!("RSI: {:.2}", rsi_value),
                    position: Point::new(5.0, y - 6.0), // Positionné à gauche, centré verticalement
                    color: Color::from_rgb(0.0, 0.8, 1.0), // Cyan pour correspondre à la ligne RSI
                    size: Pixels(11.0),
                    ..Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
    }
}

impl<Message> Program<Message> for RSIAxisProgram {
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
        let background = iced::widget::canvas::Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&background, style.background_color);

        // Le RSI varie de 0 à 100
        let max_rsi = 100.0;
        
        // Calculer le pas pour les labels (utiliser un pas fixe de 20 pour le RSI)
        let step = 20.0;
        
        // Générer les valeurs de RSI à afficher
        let mut rsi_values = Vec::new();
        let mut value = 0.0;
        while value <= max_rsi {
            rsi_values.push(value);
            value += step;
        }

        // Dessiner les labels
        for rsi_value in rsi_values {
            // Mapper le RSI (0-100) sur la hauteur du canvas
            // RSI 100 = haut (y = 0), RSI 0 = bas (y = height)
            let normalized_rsi = (rsi_value / 100.0_f64).clamp(0.0_f64, 1.0_f64);
            let y = self.height * (1.0 - normalized_rsi as f32);
            
            // S'assurer que le label est dans les bounds
            if y >= 0.0 && y <= bounds.height {
                let text = Text {
                    content: format!("{:.0}", rsi_value),
                    position: Point::new(bounds.width - 5.0 - 15.0, y), // Décaler de 15px vers la gauche
                    color: style.text_color,
                    size: Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Crée un widget canvas pour l'axe Y du RSI avec overlay de la valeur RSI actuelle
pub fn rsi_y_axis<'a>(chart_state: &'a ChartState, height: f32) -> Element<'a, crate::app::messages::Message> {
    // Calculer toutes les valeurs RSI d'abord (sur toutes les bougies, pas seulement visibles)
    let all_rsi_values = calculate_all_rsi_values(chart_state);
    
    // Récupérer la dernière valeur RSI en utilisant les valeurs pré-calculées
    let current_rsi = all_rsi_values.as_ref()
        .and_then(|values| get_last_rsi_value(chart_state, Some(values)));
    
    // Créer l'axe Y
    let axis = Canvas::new(RSIAxisProgram::new(height))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill);
    
    // Créer l'overlay avec le label RSI qui suit la valeur
    let overlay = Canvas::new(RSILabelOverlayProgram::new(height, current_rsi))
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

