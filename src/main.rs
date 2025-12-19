mod finance_chart;

use iced::widget::{button, column, container, row, text, scrollable, Space};
use iced::{Element, Length, Task, Theme, Color, Size, window, Subscription};
use finance_chart::{
    chart, load_all_from_directory, ChartState, x_axis, y_axis,
    X_AXIS_HEIGHT, Y_AXIS_WIDTH, ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    series_select_box,
    SettingsState, ChartStyle,
    settings::{color_fields, preset_colors, SerializableColor},
    ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage,
    tools_canvas::Action as HistoryAction,
};

/// Chemin vers le fichier de données
const DATA_FILE: &str = "data/BTCUSDT_1h.json";

fn main() -> iced::Result {
    iced::daemon(ChartApp::new, ChartApp::update, ChartApp::view)
        .title(ChartApp::title)
        .theme(ChartApp::theme)
        .subscription(ChartApp::subscription)
        .run()
}

/// Application principale - possède directement tout l'état (pas de Rc<RefCell>)
struct ChartApp {
    // État possédé directement
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
    
    // Gestion des fenêtres
    main_window_id: Option<window::Id>,
    settings_window_id: Option<window::Id>,
    
    // État temporaire pour l'édition des settings
    editing_style: Option<ChartStyle>,
    editing_color_index: Option<usize>,
}

/// Messages de l'application
#[derive(Debug, Clone)]
enum Message {
    // === Messages du graphique ===
    Chart(ChartMessage),
    
    // === Messages des axes ===
    YAxis(YAxisMessage),
    XAxis(XAxisMessage),
    
    // === Messages du panel d'outils ===
    ToolsPanel(ToolsPanelMessage),
    
    // === Messages du panel de séries ===
    SeriesPanel(SeriesPanelMessage),
    
    // === Messages de fenêtres ===
    OpenSettings,
    SettingsWindowOpened(window::Id),
    MainWindowOpened(window::Id),
    WindowClosed(window::Id),
    
    // === Messages des settings ===
    SelectColor(usize, SerializableColor),
    ApplySettings,
    CancelSettings,
    ToggleColorPicker(usize),
}

impl ChartApp {
    fn new() -> (Self, Task<Message>) {
        // Charger toutes les séries depuis le dossier data
        let mut chart_state = ChartState::new(1200.0, 800.0);
        
        match load_all_from_directory("data") {
            Ok(series_list) => {
                println!("✅ {} série(s) trouvée(s) dans le dossier data", series_list.len());
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
                    eprintln!("⚠️ Aucune série chargée. Vérifiez que le dossier 'data' contient des fichiers JSON.");
                }
            }
            Err(e) => {
                eprintln!("❌ Erreur lors du chargement des séries depuis 'data': {}", e);
                eprintln!("   Tentative de chargement du fichier par défaut: {}", DATA_FILE);
                // Fallback: essayer de charger le fichier par défaut
                match finance_chart::load_from_json(DATA_FILE) {
                    Ok(series) => {
                        println!("✅ Série chargée: {} bougies", series.data.len());
                        chart_state.add_series(series);
                    }
                    Err(e2) => {
                        eprintln!("❌ Erreur de chargement: {}", e2);
                        eprintln!("   Aucune donnée chargée.");
                    }
                }
            }
        }
        
        // Créer l'état des outils et charger les dessins sauvegardés
        let mut tools_state = ToolsState::default();
        if let Err(e) = tools_state.load_from_file("drawings.json") {
            if !e.to_string().contains("No such file") && !e.to_string().contains("cannot find") {
                eprintln!("⚠️ Impossible de charger les dessins: {}", e);
            }
        } else {
            println!(
                "✅ Dessins chargés: {} rectangles, {} lignes horizontales",
                tools_state.rectangles.len(),
                tools_state.horizontal_lines.len()
            );
        }

        // Charger le style
        let chart_style = match ChartStyle::load_from_file("chart_style.json") {
            Ok(style) => {
                println!("✅ Style chargé depuis chart_style.json");
                style
            }
            Err(_) => ChartStyle::default(),
        };

        // Ouvrir la fenêtre principale
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(1200.0, 800.0),
            ..Default::default()
        });

        (
            Self { 
                chart_state, 
                tools_state, 
                settings_state: SettingsState::default(),
                chart_style,
                main_window_id: Some(main_id),
                settings_window_id: None,
                editing_style: None,
                editing_color_index: None,
            },
            open_task.map(Message::MainWindowOpened),
        )
    }

    fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.settings_window_id {
            String::from("Settings - Style Chart")
        } else {
            // Afficher le symbole de la série active, ou un titre par défaut
            if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                active_series.symbol.clone()
            } else {
                String::from("CandleChart")
            }
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
            // === Gestion des messages du graphique ===
            Message::Chart(chart_msg) => {
                self.handle_chart_message(chart_msg);
                Task::none()
            }
            
            // === Gestion des messages des axes ===
            Message::YAxis(YAxisMessage::ZoomVertical { factor }) => {
                self.chart_state.zoom_vertical(factor);
                Task::none()
            }
            Message::XAxis(XAxisMessage::ZoomHorizontal { factor }) => {
                self.chart_state.zoom(factor);
                Task::none()
            }
            
            // === Gestion des messages du panel d'outils ===
            Message::ToolsPanel(ToolsPanelMessage::ToggleTool { tool }) => {
                if self.tools_state.selected_tool == Some(tool) {
                    self.tools_state.selected_tool = None;
                } else {
                    self.tools_state.selected_tool = Some(tool);
                }
                Task::none()
            }
            
            // === Gestion des messages du panel de séries ===
            Message::SeriesPanel(SeriesPanelMessage::SelectSeriesByName { series_name }) => {
                // Trouver le SeriesId correspondant au nom
                let series_id_opt = self.chart_state.series_manager.all_series()
                    .find(|s| s.full_name() == series_name)
                    .map(|s| s.id.clone());
                
                if let Some(series_id) = series_id_opt {
                    // Activer uniquement cette série (désactive toutes les autres)
                    self.chart_state.series_manager.activate_only_series(series_id);
                    // Mettre à jour le viewport après activation
                    self.chart_state.update_viewport_from_series();
                }
                Task::none()
            }
            
            // === Gestion des fenêtres ===
            Message::MainWindowOpened(_id) => Task::none(),
            
            Message::OpenSettings => {
                if self.settings_window_id.is_some() {
                    return Task::none();
                }
                self.editing_style = Some(self.chart_style.clone());
                self.editing_color_index = None;
                
                let (id, task) = window::open(window::Settings {
                    size: Size::new(500.0, 450.0),
                    resizable: false,
                    ..Default::default()
                });
                self.settings_window_id = Some(id);
                task.map(Message::SettingsWindowOpened)
            }
            
            Message::SettingsWindowOpened(_id) => Task::none(),
            
            Message::WindowClosed(id) => {
                if Some(id) == self.settings_window_id {
                    self.settings_window_id = None;
                    self.editing_style = None;
                    self.editing_color_index = None;
                } else if Some(id) == self.main_window_id {
                    self.main_window_id = None;
                    if let Some(settings_id) = self.settings_window_id {
                        return window::close(settings_id);
                    }
                }
                Task::none()
            }
            
            // === Gestion des settings ===
            Message::SelectColor(field_index, color) => {
                if let Some(ref mut style) = self.editing_style {
                    let fields = color_fields();
                    if field_index < fields.len() {
                        (fields[field_index].set)(style, color);
                    }
                }
                self.editing_color_index = None;
                Task::none()
            }
            
            Message::ApplySettings => {
                if let Some(new_style) = self.editing_style.take() {
                    self.chart_style = new_style.clone();
                    if let Err(e) = new_style.save_to_file("chart_style.json") {
                        eprintln!("⚠️ Erreur sauvegarde style: {}", e);
                    } else {
                        println!("✅ Style sauvegardé dans chart_style.json");
                    }
                }
                if let Some(id) = self.settings_window_id {
                    self.settings_window_id = None;
                    self.editing_color_index = None;
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::CancelSettings => {
                self.editing_style = None;
                self.editing_color_index = None;
                if let Some(id) = self.settings_window_id {
                    self.settings_window_id = None;
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::ToggleColorPicker(index) => {
                if self.editing_color_index == Some(index) {
                    self.editing_color_index = None;
                } else {
                    self.editing_color_index = Some(index);
                }
                Task::none()
            }
        }
    }

    /// Gère les messages du graphique
    fn handle_chart_message(&mut self, msg: ChartMessage) {
        match msg {
            // === Navigation ===
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
            
            // === Dessin de rectangles ===
            ChartMessage::StartDrawingRectangle { screen_x, screen_y, time, price } => {
                self.tools_state.drawing.start(screen_x, screen_y, time, price);
            }
            ChartMessage::UpdateDrawing { screen_x, screen_y } => {
                self.tools_state.drawing.update(screen_x, screen_y);
            }
            ChartMessage::FinishDrawingRectangle { end_time, end_price } => {
                if let Some(rect) = self.tools_state.drawing.finish(end_time, end_price) {
                    self.tools_state.history.record(HistoryAction::CreateRectangle { rect: rect.clone() });
                    let new_index = self.tools_state.rectangles.len();
                    self.tools_state.rectangles.push(rect);
                    self.tools_state.editing.selected_index = Some(new_index);
                    self.tools_state.selected_tool = None;
                }
            }
            
            // === Dessin de lignes horizontales ===
            ChartMessage::StartDrawingHLine { screen_y, price } => {
                self.tools_state.drawing.start(0.0, screen_y, 0, price);
            }
            ChartMessage::FinishDrawingHLine => {
                if let Some(line) = self.tools_state.drawing.finish_hline() {
                    self.tools_state.history.record(HistoryAction::CreateHLine { line: line.clone() });
                    let new_index = self.tools_state.horizontal_lines.len();
                    self.tools_state.horizontal_lines.push(line);
                    self.tools_state.hline_editing.selected_index = Some(new_index);
                    self.tools_state.selected_tool = None;
                }
            }
            ChartMessage::CancelDrawing => {
                self.tools_state.drawing.cancel();
            }
            
            // === Édition de rectangles ===
            ChartMessage::StartRectangleEdit { index, mode, time, price } => {
                if index < self.tools_state.rectangles.len() {
                    let rect_clone = self.tools_state.rectangles[index].clone();
                    self.tools_state.editing.start(index, mode, time, price, rect_clone);
                }
            }
            ChartMessage::UpdateRectangleEdit { time, price } => {
                if let Some(index) = self.tools_state.editing.selected_index {
                    if index < self.tools_state.rectangles.len() {
                        use finance_chart::interaction::apply_edit_update;
                        let edit_state = self.tools_state.editing.clone();
                        apply_edit_update(&mut self.tools_state.rectangles[index], &edit_state, time, price);
                    }
                }
            }
            ChartMessage::FinishRectangleEdit => {
                if let (Some(idx), Some(old_rect)) = (
                    self.tools_state.editing.selected_index,
                    self.tools_state.editing.original_rect.clone(),
                ) {
                    if idx < self.tools_state.rectangles.len() {
                        let new_rect = self.tools_state.rectangles[idx].clone();
                        if old_rect.start_time != new_rect.start_time ||
                           old_rect.end_time != new_rect.end_time ||
                           old_rect.start_price != new_rect.start_price ||
                           old_rect.end_price != new_rect.end_price {
                            self.tools_state.history.record(HistoryAction::ModifyRectangle {
                                index: idx,
                                old_rect,
                                new_rect,
                            });
                        }
                    }
                }
                self.tools_state.editing.finish();
            }
            ChartMessage::DeselectRectangle => {
                self.tools_state.editing.deselect();
            }
            
            // === Édition de lignes horizontales ===
            ChartMessage::StartHLineEdit { index, price } => {
                if index < self.tools_state.horizontal_lines.len() {
                    let line_clone = self.tools_state.horizontal_lines[index].clone();
                    self.tools_state.hline_editing.start(index, price, line_clone);
                }
            }
            ChartMessage::UpdateHLineEdit { price } => {
                if let Some(index) = self.tools_state.hline_editing.selected_index {
                    if index < self.tools_state.horizontal_lines.len() {
                        if let Some(ref original) = self.tools_state.hline_editing.original_line {
                            if let Some(start_price) = self.tools_state.hline_editing.start_price {
                                let delta = price - start_price;
                                self.tools_state.horizontal_lines[index].price = original.price + delta;
                            }
                        }
                    }
                }
            }
            ChartMessage::FinishHLineEdit => {
                if let (Some(idx), Some(old_line)) = (
                    self.tools_state.hline_editing.selected_index,
                    self.tools_state.hline_editing.original_line.clone(),
                ) {
                    if idx < self.tools_state.horizontal_lines.len() {
                        let new_line = self.tools_state.horizontal_lines[idx].clone();
                        if (old_line.price - new_line.price).abs() > 0.0001 {
                            self.tools_state.history.record(HistoryAction::ModifyHLine {
                                index: idx,
                                old_line,
                                new_line,
                            });
                        }
                    }
                }
                self.tools_state.hline_editing.finish();
            }
            ChartMessage::DeselectHLine => {
                self.tools_state.hline_editing.deselect();
            }
            
            // === Suppression ===
            ChartMessage::DeleteSelected => {
                // Supprimer rectangle sélectionné
                if let Some(index) = self.tools_state.editing.selected_index {
                    if index < self.tools_state.rectangles.len() {
                        let deleted_rect = self.tools_state.rectangles[index].clone();
                        self.tools_state.history.record(HistoryAction::DeleteRectangle { index, rect: deleted_rect });
                        self.tools_state.rectangles.remove(index);
                        self.tools_state.editing.deselect();
                        return;
                    }
                }
                
                // Supprimer ligne horizontale sélectionnée
                if let Some(index) = self.tools_state.hline_editing.selected_index {
                    if index < self.tools_state.horizontal_lines.len() {
                        let deleted_line = self.tools_state.horizontal_lines[index].clone();
                        self.tools_state.history.record(HistoryAction::DeleteHLine { index, line: deleted_line });
                        self.tools_state.horizontal_lines.remove(index);
                        self.tools_state.hline_editing.deselect();
                    }
                }
            }
            
            // === Historique ===
            ChartMessage::Undo => {
                self.tools_state.editing.deselect();
                self.tools_state.hline_editing.deselect();
                self.tools_state.history.undo(
                    &mut self.tools_state.rectangles,
                    &mut self.tools_state.horizontal_lines,
                );
            }
            ChartMessage::Redo => {
                self.tools_state.editing.deselect();
                self.tools_state.hline_editing.deselect();
                self.tools_state.history.redo(
                    &mut self.tools_state.rectangles,
                    &mut self.tools_state.horizontal_lines,
                );
            }
            
            // === Persistance ===
            ChartMessage::SaveDrawings => {
                if let Err(e) = self.tools_state.save_to_file("drawings.json") {
                    eprintln!("❌ Erreur de sauvegarde: {}", e);
                } else {
                    println!("✅ Dessins sauvegardés dans drawings.json");
                }
            }
            ChartMessage::LoadDrawings => {
                if let Err(e) = self.tools_state.load_from_file("drawings.json") {
                    eprintln!("❌ Erreur de chargement: {}", e);
                } else {
                    println!("✅ Dessins chargés depuis drawings.json");
                }
            }
            
            // === Position souris ===
            ChartMessage::MouseMoved { position } => {
                self.chart_state.interaction.mouse_position = Some(position);
            }
            
            // === Resize ===
            ChartMessage::Resize { width, height } => {
                self.chart_state.resize(width, height);
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.settings_window_id {
            self.view_settings()
        } else {
            self.view_main()
        }
    }

    fn view_main(&self) -> Element<'_, Message> {
        // Récupérer le symbole de la série active pour le titre
        let title_text = self.chart_state.series_manager
            .active_series()
            .next()
            .map(|series| series.symbol.clone())
            .unwrap_or_else(|| String::from("Chart Candlestick"));
        
        // Header avec titre et select box de séries
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

        // Ligne principale : Tools (gauche) + Chart (centre) + Axe Y (droite)
        let chart_row = row![
            tools_panel(&self.tools_state).map(Message::ToolsPanel),
            chart(&self.chart_state, &self.tools_state, &self.settings_state, &self.chart_style)
                .map(Message::Chart),
            y_axis(&self.chart_state).map(Message::YAxis)
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Ligne du bas : espace vide (sous tools) + Axe X + bouton settings (coin)
        let x_axis_row = row![
            container(text("")).width(Length::Fixed(TOOLS_PANEL_WIDTH)),
            x_axis(&self.chart_state).map(Message::XAxis),
            corner_settings_button()
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

    fn view_settings(&self) -> Element<'_, Message> {
        let fields = color_fields();
        let presets = preset_colors();
        
        let editing_style = self.editing_style.as_ref();
        
        // Titre
        let title = text("Style du graphique")
            .size(20)
            .color(Color::WHITE);

        // Séparateur
        let separator = || container(Space::new().height(1))
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.35))),
                ..Default::default()
            });

        // Liste des champs de couleur
        let mut color_rows = column![].spacing(10);
        
        for (index, field) in fields.iter().enumerate() {
            let current_color = if let Some(style) = editing_style {
                (field.get)(style)
            } else {
                SerializableColor::from_iced(Color::WHITE)
            };
            
            let color_box = container(text(""))
                .width(Length::Fixed(30.0))
                .height(Length::Fixed(25.0))
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(current_color.to_iced())),
                    border: iced::Border {
                        color: Color::WHITE,
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                });

            let color_btn = button(color_box)
                .on_press(Message::ToggleColorPicker(index))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..Default::default()
                });

            let label = text(field.label)
                .size(14)
                .color(Color::from_rgb(0.8, 0.8, 0.8));

            let field_row = row![
                label,
                Space::new().width(Length::Fill),
                color_btn
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center);

            color_rows = color_rows.push(field_row);

            // Si ce color picker est ouvert, afficher les presets
            if self.editing_color_index == Some(index) {
                let mut presets_row = row![].spacing(5);
                for preset in &presets {
                    let preset_color = *preset;
                    let preset_box = container(text(""))
                        .width(Length::Fixed(24.0))
                        .height(Length::Fixed(24.0))
                        .style(move |_theme| container::Style {
                            background: Some(iced::Background::Color(preset_color.to_iced())),
                            border: iced::Border {
                                color: Color::from_rgb(0.5, 0.5, 0.5),
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            ..Default::default()
                        });
                    
                    let preset_btn = button(preset_box)
                        .on_press(Message::SelectColor(index, preset_color))
                        .padding(0)
                        .style(|_theme, _status| button::Style {
                            background: None,
                            ..Default::default()
                        });
                    
                    presets_row = presets_row.push(preset_btn);
                }
                
                let presets_container = container(
                    scrollable(presets_row).direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default().width(5).scroller_width(5)
                    ))
                )
                .padding(10)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.25))),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.35),
                        width: 1.0,
                        radius: 5.0.into(),
                    },
                    ..Default::default()
                });
                
                color_rows = color_rows.push(presets_container);
            }
        }

        // Boutons Apply/Cancel
        let apply_btn = button(
            text("Appliquer").size(14)
        )
        .on_press(Message::ApplySettings)
        .padding([8, 20])
        .style(|_theme, _status| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.2))),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let cancel_btn = button(
            text("Annuler").size(14)
        )
        .on_press(Message::CancelSettings)
        .padding([8, 20])
        .style(|_theme, _status| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.5, 0.2, 0.2))),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let buttons_row = row![
            Space::new().width(Length::Fill),
            cancel_btn,
            apply_btn
        ]
        .spacing(10);

        // Layout complet
        let content = column![
            title,
            Space::new().height(10),
            separator(),
            Space::new().height(10),
            scrollable(color_rows).height(Length::Fill),
            Space::new().height(10),
            separator(),
            Space::new().height(10),
            buttons_row
        ]
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
                ..Default::default()
            })
            .into()
    }
}

/// Bouton settings dans le coin (version qui envoie un message)
fn corner_settings_button<'a>() -> Element<'a, Message> {
    let icon = text("⚙").size(18);
    
    button(
        container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
    .width(Length::Fixed(Y_AXIS_WIDTH))
    .height(Length::Fixed(X_AXIS_HEIGHT))
    .on_press(Message::OpenSettings)
    .style(|_theme, status| {
        let bg_color = match status {
            button::Status::Hovered => Color::from_rgb(0.2, 0.2, 0.25),
            _ => Color::from_rgb(0.15, 0.15, 0.18),
        };
        button::Style {
            background: Some(iced::Background::Color(bg_color)),
            text_color: Color::WHITE,
            ..Default::default()
        }
    })
    .into()
}
