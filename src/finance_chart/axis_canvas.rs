//! Canvas séparés pour les axes X et Y du graphique
//! 
//! Architecture Elm : émet des messages pour les mutations d'état,
//! reçoit des références immuables pour le rendu.

use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Text, Action};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size};
use iced::mouse;

use super::state::ChartState;
use super::render::{calculate_nice_step, calculate_nice_time_step, format_time};
use super::messages::{YAxisMessage, XAxisMessage};

/// Largeur du canvas Y (axe des prix)
pub const Y_AXIS_WIDTH: f32 = 43.0;

/// Hauteur du canvas X (axe du temps)
pub const X_AXIS_HEIGHT: f32 = 30.0;

/// Style pour les axes
pub struct AxisStyle {
    pub text_color: Color,
    pub text_size: f32,
    pub background_color: Color,
}

impl Default for AxisStyle {
    fn default() -> Self {
        Self {
            text_color: Color::from_rgba(0.8, 0.8, 0.8, 1.0),
            text_size: 11.0,
            background_color: Color::from_rgb(0.08, 0.08, 0.10),
        }
    }
}

// ============================================================================
// Canvas Y (Axe des prix - à droite)
// ============================================================================

/// État local du widget Y pour le drag (UI uniquement)
#[derive(Debug, Clone, Default)]
pub struct YAxisState {
    /// Position Y de départ du drag
    drag_start_y: Option<f32>,
    /// Est-ce qu'on est en train de drag
    is_dragging: bool,
}

/// Program pour l'axe Y (prix)
/// Reçoit une référence immutable, émet des messages
pub struct YAxisProgram<'a> {
    chart_state: &'a ChartState,
}

impl<'a> YAxisProgram<'a> {
    pub fn new(chart_state: &'a ChartState) -> Self {
        Self { chart_state }
    }
}

impl<'a> Program<YAxisMessage> for YAxisProgram<'a> {
    type State = YAxisState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let style = AxisStyle::default();

        // Fond de l'axe
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            style.background_color,
        );

        let viewport = &self.chart_state.viewport;

        // Calculer les niveaux de prix
        let (min_price, max_price) = viewport.price_scale().price_range();
        let price_range = max_price - min_price;
        let price_step = calculate_nice_step(price_range);

        // Trouver le premier niveau rond >= min_price
        let first_price = (min_price / price_step).ceil() * price_step;

        let mut price = first_price;
        while price <= max_price {
            let y = viewport.price_scale().price_to_y(price);

            // Ne dessiner que si visible
            if y >= 0.0 && y <= viewport.height() {
                // Formater le prix
                let label = if price_step >= 1.0 {
                    format!("{:.0}", price)
                } else if price_step >= 0.1 {
                    format!("{:.1}", price)
                } else {
                    format!("{:.2}", price)
                };

                let text = Text {
                    content: label,
                    position: Point::new(5.0, y - 6.0),
                    color: style.text_color,
                    size: iced::Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }

            price += price_step;
        }

        // === Dessiner le rectangle du prix courant ===
        if let Some(last_candle) = self.chart_state.last_candle() {
            let current_price = last_candle.close;
            let y = viewport.price_scale().price_to_y(current_price);
            
            // Ne dessiner que si visible
            if y >= 0.0 && y <= viewport.height() {
                // Couleur selon si le prix est haussier ou baissier
                let is_bullish = last_candle.close >= last_candle.open;
                let bg_color = if is_bullish {
                    Color::from_rgb(0.0, 0.5, 0.0) // Vert foncé
                } else {
                    Color::from_rgb(0.5, 0.0, 0.0) // Rouge foncé
                };
                
                // Dimensions du rectangle
                let rect_height = 16.0;
                let rect_width = bounds.width - 4.0;
                let rect_x = 2.0;
                let rect_y = y - rect_height / 2.0;
                
                // Dessiner le rectangle de fond
                let rect = Path::rectangle(
                    Point::new(rect_x, rect_y),
                    Size::new(rect_width, rect_height),
                );
                frame.fill(&rect, bg_color);
                
                // Formater et afficher le prix
                let label = format!("{:.2}", current_price);
                let text = Text {
                    content: label,
                    position: Point::new(5.0, y - 5.0),
                    color: Color::WHITE,
                    size: iced::Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        axis_state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<YAxisMessage>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    axis_state.is_dragging = true;
                    axis_state.drag_start_y = Some(position.y);
                    return Some(Action::request_redraw());
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                axis_state.is_dragging = false;
                axis_state.drag_start_y = None;
                return Some(Action::request_redraw());
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if axis_state.is_dragging {
                    let current_y = position.y - bounds.y;
                    
                    if let Some(start_y) = axis_state.drag_start_y {
                        let delta_y = current_y - start_y;
                        
                        // Calculer le facteur de zoom basé sur le déplacement
                        // Vers le haut (delta négatif) = zoom in, vers le bas = zoom out
                        let zoom_factor = if delta_y.abs() > 2.0 {
                            if delta_y > 0.0 {
                                1.02 // Zoom out
                            } else {
                                0.98 // Zoom in
                            }
                        } else {
                            1.0
                        };
                        
                        if zoom_factor != 1.0 {
                            // Mettre à jour la position de départ pour le prochain mouvement
                            axis_state.drag_start_y = Some(current_y);
                            // Émettre le message de zoom
                            return Some(Action::publish(YAxisMessage::ZoomVertical { factor: zoom_factor }));
                        }
                    }
                    
                    return Some(Action::request_redraw());
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
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // Changer le curseur si la souris est sur l'axe Y
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un élément canvas pour l'axe Y
/// Retourne un Element qui émet YAxisMessage
pub fn y_axis<'a>(chart_state: &'a ChartState) -> Element<'a, YAxisMessage> {
    Canvas::new(YAxisProgram::new(chart_state))
        .width(Length::Fixed(Y_AXIS_WIDTH))
        .height(Length::Fill)
        .into()
}

// ============================================================================
// Canvas X (Axe du temps - en bas)
// ============================================================================

/// État local du widget X pour le drag (UI uniquement)
#[derive(Debug, Clone, Default)]
pub struct XAxisState {
    /// Position X de départ du drag
    drag_start_x: Option<f32>,
    /// Est-ce qu'on est en train de drag
    is_dragging: bool,
}

/// Program pour l'axe X (temps)
/// Reçoit une référence immutable, émet des messages
pub struct XAxisProgram<'a> {
    chart_state: &'a ChartState,
}

impl<'a> XAxisProgram<'a> {
    pub fn new(chart_state: &'a ChartState) -> Self {
        Self { chart_state }
    }
}

impl<'a> Program<XAxisMessage> for XAxisProgram<'a> {
    type State = XAxisState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let style = AxisStyle::default();

        // Fond de l'axe
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            style.background_color,
        );

        let viewport = &self.chart_state.viewport;

        // Calculer les timestamps
        let (min_time, max_time) = viewport.time_scale().time_range();
        let time_range = max_time - min_time;
        let time_step = calculate_nice_time_step(time_range);

        // Trouver le premier timestamp rond >= min_time
        let first_time = ((min_time / time_step) + 1) * time_step;

        let mut time = first_time;
        while time <= max_time {
            let x = viewport.time_scale().time_to_x(time);

            // Ne dessiner que si visible
            if x >= 0.0 && x <= viewport.width() {
                let label = format_time(time, time_step);

                let text = Text {
                    content: label,
                    position: Point::new(x - 15.0, 8.0),
                    color: style.text_color,
                    size: iced::Pixels(style.text_size),
                    ..Text::default()
                };
                frame.fill_text(text);
            }

            time += time_step;
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        axis_state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<XAxisMessage>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    axis_state.is_dragging = true;
                    axis_state.drag_start_x = Some(position.x);
                    return Some(Action::request_redraw());
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                axis_state.is_dragging = false;
                axis_state.drag_start_x = None;
                return Some(Action::request_redraw());
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if axis_state.is_dragging {
                    let current_x = position.x - bounds.x;
                    
                    if let Some(start_x) = axis_state.drag_start_x {
                        let delta_x = current_x - start_x;
                        
                        // Calculer le facteur de zoom basé sur le déplacement
                        // Vers la droite (delta positif) = zoom out, vers la gauche = zoom in
                        let zoom_factor = if delta_x.abs() > 2.0 {
                            if delta_x > 0.0 {
                                1.02 // Zoom out
                            } else {
                                0.98 // Zoom in
                            }
                        } else {
                            1.0
                        };
                        
                        if zoom_factor != 1.0 {
                            // Mettre à jour la position de départ pour le prochain mouvement
                            axis_state.drag_start_x = Some(current_x);
                            // Émettre le message de zoom
                            return Some(Action::publish(XAxisMessage::ZoomHorizontal { factor: zoom_factor }));
                        }
                    }
                    
                    return Some(Action::request_redraw());
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
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // Changer le curseur si la souris est sur l'axe X
        if cursor.is_over(bounds) {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Crée un élément canvas pour l'axe X
/// Retourne un Element qui émet XAxisMessage
pub fn x_axis<'a>(chart_state: &'a ChartState) -> Element<'a, XAxisMessage> {
    Canvas::new(XAxisProgram::new(chart_state))
        .width(Length::Fill)
        .height(Length::Fixed(X_AXIS_HEIGHT))
        .into()
}
