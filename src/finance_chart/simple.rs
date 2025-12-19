//! API simplifiée pour une utilisation rapide du crate
//!
//! Cette module fournit des fonctions helper pour créer une application
//! de graphique en quelques lignes de code, tout en gardant la possibilité
//! de personnaliser via l'API avancée si nécessaire.

use iced::{Element, Length, Task, Theme, Color, Size, window, Subscription};
use iced::widget::{column, row, container, text, Space};

use super::{
    ChartState, chart, load_all_from_directory,
    ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    ChartStyle, SettingsState,
    ChartMessage, SeriesPanelMessage,
    x_axis, y_axis, X_AXIS_HEIGHT,
    series_select_box,
};

/// Application simplifiée qui encapsule toute la logique
pub struct SimpleChartApp {
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
    main_window_id: Option<window::Id>,
    title: String,
}

impl SimpleChartApp {
    /// Crée une nouvelle application simplifiée
    pub fn new(data_path: impl AsRef<str>, width: f32, height: f32) -> Self {
        let mut chart_state = ChartState::new(width, height);
        
        // Charger automatiquement toutes les séries
        match load_all_from_directory(data_path.as_ref()) {
            Ok(series_list) => {
                for series in series_list {
                    chart_state.add_series(series);
                }
            }
            Err(e) => {
                eprintln!("⚠️ Erreur lors du chargement des données: {}", e);
            }
        }
        
        // Déterminer le titre depuis la série active
        let title = chart_state.series_manager
            .active_series()
            .next()
            .map(|s| s.symbol.clone())
            .unwrap_or_else(|| String::from("CandleChart"));
        
        Self {
            chart_state,
            tools_state: ToolsState::default(),
            settings_state: SettingsState::default(),
            chart_style: ChartStyle::default(),
            main_window_id: None,
            title,
        }
    }
    
    /// Définit un titre personnalisé
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    
    /// Charge un style personnalisé depuis un fichier
    pub fn with_style_file(mut self, path: impl AsRef<str>) -> Self {
        if let Ok(style) = ChartStyle::load_from_file(path.as_ref()) {
            self.chart_style = style;
        }
        self
    }
    
    /// Charge les dessins depuis un fichier
    pub fn with_drawings_file(mut self, path: impl AsRef<str>) -> Self {
        if let Err(e) = self.tools_state.load_from_file(path.as_ref()) {
            if !e.to_string().contains("No such file") && !e.to_string().contains("cannot find") {
                eprintln!("⚠️ Erreur lors du chargement des dessins: {}", e);
            }
        }
        self
    }
    
    /// Accède à l'état du graphique pour personnalisation avancée
    pub fn chart_state_mut(&mut self) -> &mut ChartState {
        &mut self.chart_state
    }
    
    /// Accède à l'état du graphique (lecture seule)
    pub fn chart_state(&self) -> &ChartState {
        &self.chart_state
    }
    
    /// Accède à l'état des outils pour personnalisation avancée
    pub fn tools_state_mut(&mut self) -> &mut ToolsState {
        &mut self.tools_state
    }
    
    /// Accède au style pour personnalisation avancée
    pub fn style_mut(&mut self) -> &mut ChartStyle {
        &mut self.chart_style
    }
}

/// Messages internes de l'application simplifiée
#[derive(Debug, Clone)]
pub enum Message {
    Chart(ChartMessage),
    SeriesPanel(SeriesPanelMessage),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl SimpleChartApp {
    /// Retourne le titre de la fenêtre
    pub fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.main_window_id {
            if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                format!("{} - {}", self.title, active_series.symbol)
            } else {
                self.title.clone()
            }
        } else {
            self.title.clone()
        }
    }
    
    /// Retourne le thème
    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }
    
    /// Retourne la subscription
    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }
    
    /// Met à jour l'état de l'application
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Chart(chart_msg) => {
                self.handle_chart_message(chart_msg);
                Task::none()
            }
            
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                let series_id_opt = self.chart_state.series_manager.all_series()
                    .find(|s| s.full_name() == series_name)
                    .map(|s| s.id.clone());
                
                if let Some(series_id) = series_id_opt {
                    self.chart_state.series_manager.activate_only_series(series_id);
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
    
    /// Retourne la vue de l'application
    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
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
}

impl SimpleChartApp {
    /// Gère les messages du graphique
    fn handle_chart_message(&mut self, msg: ChartMessage) {
        match msg {
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
            ChartMessage::Resize { width, height } => {
                self.chart_state.resize(width, height);
            }
            ChartMessage::MouseMoved { position: _ } => {
                // Géré automatiquement par le widget
            }
            _ => {
                // Autres messages non gérés dans la version simple
                // Peuvent être gérés via l'API avancée
            }
        }
    }
    
    fn view_main(&self) -> Element<'_, Message> {
        let title_text = self.chart_state.series_manager
            .active_series()
            .next()
            .map(|series| series.symbol.clone())
            .unwrap_or_else(|| self.title.clone());
        
        let header = container(
            row![
                text(title_text)
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
        
        let chart_row = row![
            tools_panel(&self.tools_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 })),
            chart(&self.chart_state, &self.tools_state, &self.settings_state, &self.chart_style)
                .map(Message::Chart),
            y_axis(&self.chart_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 }))
        ]
        .width(Length::Fill)
        .height(Length::Fill);
        
        let x_axis_row = row![
            container(text("")).width(Length::Fixed(TOOLS_PANEL_WIDTH)),
            x_axis(&self.chart_state).map(|_| Message::Chart(ChartMessage::Resize { width: 0.0, height: 0.0 }))
        ]
        .width(Length::Fill)
        .height(Length::Fixed(X_AXIS_HEIGHT));
        
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

/// Crée et lance une application simplifiée
///
/// # Exemple
///
/// ```no_run
/// use candlechart::simple_app;
///
/// fn main() -> iced::Result {
///     simple_app("data", 1200.0, 800.0)
/// }
/// ```
pub fn simple_app(data_path: impl AsRef<str>, width: f32, height: f32) -> iced::Result {
    let data_path_str = data_path.as_ref().to_string();
    
    // Fonction d'initialisation
    let init = move || {
        let mut app = SimpleChartApp::new(&data_path_str, width, height);
        
        // Ouvrir la fenêtre principale
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(width, height),
            ..Default::default()
        });
        
        app.main_window_id = Some(main_id);
        (app, open_task.map(Message::MainWindowOpened))
    };
    
    iced::daemon(init, SimpleChartApp::update, SimpleChartApp::view)
        .title(|app: &SimpleChartApp, window_id| app.title(window_id))
        .theme(|app: &SimpleChartApp, window_id| app.theme(window_id))
        .subscription(|app: &SimpleChartApp| app.subscription())
        .run()
}

