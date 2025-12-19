//! Exemple d'utilisation avancée du crate CandleChart
//!
//! Cet exemple montre comment utiliser l'API avancée pour une personnalisation complète.

use iced::widget::{column, row, container, text, Space};
use iced::{Element, Length, Task, Theme, Color, Size, window, Subscription};
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

struct TestApp {
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
    main_window_id: Option<window::Id>,
}

#[derive(Debug, Clone)]
enum Message {
    Chart(ChartMessage),
    SeriesPanel(SeriesPanelMessage),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl TestApp {
    fn new() -> (Self, Task<Message>) {
        let mut chart_state = ChartState::new(1200.0, 800.0);
        
        if let Ok(series_list) = load_all_from_directory("../data") {
            for series in series_list {
                chart_state.add_series(series);
            }
        }
        
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(1200.0, 800.0),
            ..Default::default()
        });
        
        (
            Self {
                chart_state,
                tools_state: ToolsState::default(),
                settings_state: SettingsState::default(),
                chart_style: ChartStyle::default(),
                main_window_id: Some(main_id),
            },
            open_task.map(Message::MainWindowOpened),
        )
    }
    
    fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.main_window_id {
            if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                format!("Test - {}", active_series.symbol)
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
    
    fn handle_chart_message(&mut self, msg: ChartMessage) {
        match msg {
            ChartMessage::StartPan { position } => self.chart_state.start_pan(position),
            ChartMessage::UpdatePan { position } => self.chart_state.update_pan(position),
            ChartMessage::EndPan => self.chart_state.end_pan(),
            ChartMessage::ZoomHorizontal { factor } => self.chart_state.zoom(factor),
            ChartMessage::ZoomVertical { factor } => self.chart_state.zoom_vertical(factor),
            ChartMessage::ZoomBoth { factor } => self.chart_state.zoom_both(factor),
            ChartMessage::Resize { width, height } => self.chart_state.resize(width, height),
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
        let title_text = self.chart_state.series_manager
            .active_series()
            .next()
            .map(|series| series.symbol.clone())
            .unwrap_or_else(|| String::from("Test CandleChart"));
        
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

