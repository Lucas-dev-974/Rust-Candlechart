//! Widget Canvas pour afficher les volumes échangés
//!
//! Affiche des barres de volume pour chaque bougie, synchronisé avec le graphique principal.

use iced::widget::canvas::{Canvas, Frame, Geometry, Program, Path};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced::mouse::Cursor;

use super::state::ChartState;
use super::scale::VolumeScale;
use super::render::{calculate_bar_width, calculate_candle_period};

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

impl<'a, Message> Program<Message> for VolumeProgram<'a> {
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
        
        // Créer un TimeScale temporaire pour le volume chart qui utilise bounds.width
        // Cela garantit que les positions X sont calculées correctement pour ce canvas
        let (min_time, max_time) = viewport.time_scale().time_range();
        use super::scale::TimeScale;
        let volume_time_scale = TimeScale::new(min_time, max_time, bounds.width);

        // Calculer la largeur des barres via le module bar_sizing
        let candle_period = calculate_candle_period(candles);
        let bar_width = calculate_bar_width(candle_period, max_time - min_time, bounds.width);

        // Dessiner les barres de volume
        // Utiliser le volume_time_scale qui utilise bounds.width pour calculer les positions X
        for candle in candles.iter() {
            // Vérifier que le volume est valide
            if candle.volume.is_nan() || candle.volume < 0.0 {
                continue;
            }
            
            // Calculer la position X directement avec le volume_time_scale
            let x = volume_time_scale.time_to_x(candle.timestamp);
            
            // Vérifier si la barre est visible horizontalement dans le canvas du volume chart
            // Utiliser une marge très large pour s'assurer que toutes les barres visibles sont dessinées
            // Même si elles sont partiellement hors écran
            if x >= -bar_width * 3.0 && x <= bounds.width + bar_width * 3.0 {
                // Utiliser volume_to_y pour calculer la position Y du haut de la barre
                // Cela garantit la cohérence avec l'axe Y qui utilise aussi volume_to_y
                let y_bottom = bounds.height;
                let y_top = self.volume_scale.volume_to_y(candle.volume);
                
                // La hauteur de la barre est la distance entre le bas et le haut
                // volume_to_y retourne la position Y du haut de la barre
                let bar_height = (y_bottom - y_top).max(0.0);

                // Dessiner la barre même si la hauteur est très petite (pour les volumes proches de 0)
                // Mais s'assurer qu'elle est au moins visible (minimum 1 pixel)
                if bar_height >= 0.0 {
                    // Couleur de la barre : verte si haussière, rouge si baissière
                    let bar_color = if candle.is_bullish() {
                        Color::from_rgba(0.0, 0.6, 0.0, 0.7) // Vert avec transparence
                    } else {
                        Color::from_rgba(0.8, 0.0, 0.0, 0.7) // Rouge avec transparence
                    };

                    // S'assurer que la barre a au moins une hauteur minimale pour être visible
                    let final_height = bar_height.max(1.0);
                    let final_y_top = y_bottom - final_height;

                    // Dessiner la barre de volume depuis y_top jusqu'à y_bottom
                    let bar = Path::rectangle(
                        Point::new(x - bar_width / 2.0, final_y_top),
                        Size::new(bar_width, final_height),
                    );
                    frame.fill(&bar, bar_color);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Crée un élément canvas pour afficher les volumes
pub fn volume_chart<'a, Message: 'a>(
    chart_state: &'a ChartState,
    volume_scale: VolumeScale,
) -> Element<'a, Message> {
    Canvas::new(VolumeProgram::new(chart_state, volume_scale))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

