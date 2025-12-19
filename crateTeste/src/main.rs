//! Test du crate CandleChart
//!
//! Ce projet teste l'utilisation de CandleChart comme librairie externe.

use iced::widget::{column, row, container, text, Space};
use iced::{Element, Length, Task, Theme, Color, Size, window, Subscription};
// Utiliser le nom de la librairie (candlechart) défini dans [lib] du Cargo.toml parent
use candlechart::{
    ChartState, chart, load_all_from_directory,
    ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    ChartStyle, SettingsState,
    ChartMessage, SeriesPanelMessage,
    x_axis, y_axis, X_AXIS_HEIGHT,
    series_select_box,
};

fn main() -> iced::Result {
    iced::daemon(TestApp::new, TestApp::update, TestApp::view)
        .title(TestApp::title)
        .theme(TestApp::theme)
        .subscription(TestApp::subscription)
        .run()
}

/// Application de test
struct TestApp {
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
    main_window_id: Option<window::Id>,
}

/// Messages de l'application de test
#[derive(Debug, Clone)]
enum Message {
    Chart(ChartMessage),
    SeriesPanel(SeriesPanelMessage),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl TestApp {
    fn new() -> (Self, Task<Message>) {
        println!("🚀 Initialisation de l'application de test du crate CandleChart");
        
        // Créer l'état du graphique
        let mut chart_state = ChartState::new(1200.0, 800.0);
        
        // Charger toutes les séries depuis le dossier data du projet parent
        match load_all_from_directory("../data") {
            Ok(series_list) => {
                println!("✅ {} série(s) chargée(s) depuis ../data", series_list.len());
                for series in series_list {
                    let series_name = series.full_name();
                    println!(
                        "  📊 {}: {} bougies ({} - {})",
                        series_name,
                        series.data.len(),
                        series.symbol,
                        series.interval
                    );
                    chart_state.add_series(series);
                }
                if chart_state.series_manager.total_count() == 0 {
                    eprintln!("⚠️ Aucune série chargée. Vérifiez que le dossier '../data' contient des fichiers JSON.");
                }
            }
            Err(e) => {
                eprintln!("❌ Erreur lors du chargement des séries: {}", e);
            }
        }
        
        // Créer les états
        let tools_state = ToolsState::default();
        let settings_state = SettingsState::default();
        let chart_style = ChartStyle::default();
        
        // Ouvrir la fenêtre principale
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(1200.0, 800.0),
            ..Default::default()
        });
        
        (
            Self {
                chart_state,
                tools_state,
                settings_state,
                chart_style,
                main_window_id: Some(main_id),
            },
            open_task.map(Message::MainWindowOpened),
        )
    }
    
    fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.main_window_id {
            // Afficher le symbole de la série active
            if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                format!("Test CandleChart - {}", active_series.symbol)
            } else {
                String::from("Test CandleChart")
            }
        } else {
            String::from("Test CandleChart")
        }
    }
    
    fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }
    
    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
    
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Chart(chart_msg) => {
                self.handle_chart_message(chart_msg);
                Task::none()
            }
            
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                // Trouver le SeriesId correspondant au nom
                let series_id_opt = self.chart_state.series_manager.all_series()
                    .find(|s| s.full_name() == series_name)
                    .map(|s| s.id.clone());
                
                if let Some(series_id) = series_id_opt {
                    // Activer uniquement cette série
                    self.chart_state.series_manager.activate_only_series(series_id);
                    // Mettre à jour le viewport
                    self.chart_state.update_viewport_from_series();
                }
                Task::none()
            }
            
            Message::MainWindowOpened(_id) => Task::none(),
            
            Message::WindowClosed(id) => {
                if Some(id) == self.main_window_id {
                    self.main_window_id = None;
                }
                Task::none()
            }
        }
    }
    
    /// Gère les messages du graphique
    fn handle_chart_message(&mut self, msg: ChartMessage) {
        match msg {
            // Navigation
            ChartMessage::StartPan { position } => {
                self.chart_state.start_pan(position);
            }
            ChartMessage::UpdatePan { position } => {
                self.chart_state.update_pan(position);
            }
            ChartMessage::EndPan => {
                self.chart_state.end_pan();
            }
            ChartMessage::ZoomHorizontal { factor } => {
                self.chart_state.zoom(factor);
            }
            ChartMessage::ZoomVertical { factor } => {
                self.chart_state.zoom_vertical(factor);
            }
            ChartMessage::ZoomBoth { factor } => {
                self.chart_state.zoom_both(factor);
            }
            
            // Resize
            ChartMessage::Resize { width, height } => {
                self.chart_state.resize(width, height);
            }
            
            // Autres messages (non gérés dans ce test minimal)
            _ => {}
        }
    }
    
    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.main_window_id {
            self.view_main()
        } else {
            container(text("Fenêtre inconnue"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    }
    
    fn view_main(&self) -> Element<'_, Message> {
        // Récupérer le symbole de la série active pour le titre
        let title_text = self.chart_state.series_manager
            .active_series()
            .next()
            .map(|series| series.symbol.clone())
            .unwrap_or_else(|| String::from("Test CandleChart"));
        
        // Header avec titre et select box de séries
        let header = container(
            row![
                text(format!("Test: {}", title_text))
                    .size(24)
                    .color(Color::WHITE),
                Space::new().width(Length::Fill),
                series_select_box(&self.chart_state.series_manager).map(Message::SeriesPanel)
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .padding(15)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
            ..Default::default()
        });
        
        // Ligne principale : Tools (gauche) + Chart (centre) + Axe Y (droite)
        let chart_row = row![
            tools_panel(&self.tools_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 })),
            chart(&self.chart_state, &self.tools_state, &self.settings_state, &self.chart_style)
                .map(Message::Chart),
            y_axis(&self.chart_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 }))
        ]
        .width(Length::Fill)
        .height(Length::Fill);
        
        // Ligne du bas : espace vide (sous tools) + Axe X
        let x_axis_row = row![
            container(text("")).width(Length::Fixed(TOOLS_PANEL_WIDTH)),
            x_axis(&self.chart_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 }))
        ]
        .width(Length::Fill)
        .height(Length::Fixed(X_AXIS_HEIGHT));
        
        // Layout complet
        column![
            header,
            chart_row,
            x_axis_row
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
