mod finance_chart;

use iced::widget::{button, column, container, row, text, scrollable, Space, checkbox};
use iced::{Element, Length, Task, Theme, Color, Size, window, Subscription};
use std::time::Duration;
use finance_chart::{
    chart, load_all_from_directory, ChartState, x_axis, y_axis,
    X_AXIS_HEIGHT, Y_AXIS_WIDTH, ToolsState, tools_panel, TOOLS_PANEL_WIDTH,
    series_select_box,
    SettingsState, ChartStyle,
    settings::{color_fields, preset_colors, SerializableColor},
    ChartMessage, YAxisMessage, XAxisMessage, ToolsPanelMessage, SeriesPanelMessage,
    tools_canvas::Action as HistoryAction,
    BinanceProvider, UpdateResult,
    core::{SeriesId, Candle},
};

/// Chemin vers le fichier de données
const DATA_FILE: &str = "data/BTCUSDT_1h.json";

/// Dimensions par défaut de la fenêtre principale
const MAIN_WINDOW_WIDTH: f32 = 1200.0;
const MAIN_WINDOW_HEIGHT: f32 = 800.0;

/// Dimensions de la fenêtre de settings
const SETTINGS_WINDOW_WIDTH: f32 = 500.0;
const SETTINGS_WINDOW_HEIGHT: f32 = 450.0;

/// Intervalle de mise à jour en temps réel (en secondes)
const REALTIME_UPDATE_INTERVAL_SECS: f64 = 0.9;

/// Calcule le timestamp pour récupérer N bougies selon l'intervalle
fn calculate_candles_back_timestamp(interval: &str, count: usize) -> i64 {
    let seconds_per_candle = match interval {
        "1m" => 60,
        "3m" => 180,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "2h" => 7200,
        "4h" => 14400,
        "6h" => 21600,
        "8h" => 28800,
        "12h" => 43200,
        "1d" => 86400,
        "3d" => 259200,
        "1w" => 604800,
        "1M" => 2592000, // Approximation (30 jours)
        _ => 3600, // Défaut: 1h
    };
    (count * seconds_per_candle) as i64
}

fn main() -> iced::Result {
    iced::daemon(ChartApp::new, ChartApp::update, ChartApp::view)
        .title(ChartApp::title)
        .theme(ChartApp::theme)
        .subscription(ChartApp::subscription)
        .run()
}

/// Type de fenêtre
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowType {
    Main,
    Settings,
}

/// Gestionnaire de fenêtres simplifié
#[derive(Debug, Clone)]
struct WindowManager {
    main_window_id: Option<window::Id>,
    settings_window_id: Option<window::Id>,
}

impl WindowManager {
    fn new(main_id: window::Id) -> Self {
        Self {
            main_window_id: Some(main_id),
            settings_window_id: None,
        }
    }
    
    fn get_id(&self, window_type: WindowType) -> Option<window::Id> {
        match window_type {
            WindowType::Main => self.main_window_id,
            WindowType::Settings => self.settings_window_id,
        }
    }
    
    fn set_id(&mut self, window_type: WindowType, id: window::Id) {
        match window_type {
            WindowType::Main => self.main_window_id = Some(id),
            WindowType::Settings => self.settings_window_id = Some(id),
        }
    }
    
    fn remove_id(&mut self, window_type: WindowType) {
        match window_type {
            WindowType::Main => self.main_window_id = None,
            WindowType::Settings => self.settings_window_id = None,
        }
    }
    
    fn is_open(&self, window_type: WindowType) -> bool {
        self.get_id(window_type).is_some()
    }
    
    fn get_window_type(&self, id: window::Id) -> Option<WindowType> {
        if self.main_window_id == Some(id) {
            Some(WindowType::Main)
        } else if self.settings_window_id == Some(id) {
            Some(WindowType::Settings)
        } else {
            None
        }
    }
}

/// Application principale - possède directement tout l'état (pas de Rc<RefCell>)
struct ChartApp {
    // État possédé directement
    chart_state: ChartState,
    tools_state: ToolsState,
    settings_state: SettingsState,
    chart_style: ChartStyle,
    
    // Gestion des fenêtres
    windows: WindowManager,
    
    // État temporaire pour l'édition des settings
    editing_style: Option<ChartStyle>,
    editing_color_index: Option<usize>,
    
    // Mode temps réel
    binance_provider: BinanceProvider,
    realtime_enabled: bool,
    
    // Compteur de version pour forcer le re-render du canvas
    render_version: u64,
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
    ToggleAutoScroll,
    
    // === Messages temps réel ===
    RealtimeUpdate,
    RealtimeUpdateComplete(Vec<(SeriesId, String, Result<Option<Candle>, String>)>),
    CompleteMissingData,
}

impl ChartApp {
    fn new() -> (Self, Task<Message>) {
        // Charger toutes les séries depuis le dossier data
        let mut chart_state = ChartState::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT);
        
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
                        eprintln!("   Détails: {}", e2);
                    }
                }
            }
        }
        
        // Créer l'état des outils et charger les dessins sauvegardés
        let mut tools_state = ToolsState::default();
        match tools_state.load_from_file("drawings.json") {
            Ok(()) => {
                println!(
                    "✅ Dessins chargés: {} rectangles, {} lignes horizontales",
                    tools_state.rectangles.len(),
                    tools_state.horizontal_lines.len()
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                // Ignorer seulement les erreurs "fichier non trouvé"
                if !error_msg.contains("No such file") 
                    && !error_msg.contains("cannot find")
                    && !error_msg.contains("not found") {
                    eprintln!("⚠️ Impossible de charger les dessins: {}", e);
                }
            }
        }

        // Charger le style
        let chart_style = match ChartStyle::load_from_file("chart_style.json") {
            Ok(style) => {
                println!("✅ Style chargé depuis chart_style.json");
                style
            }
            Err(_) => ChartStyle::default(),
        };

        // Créer le provider Binance pour le mode temps réel
        let binance_provider = BinanceProvider::new();
        
        // Compléter les données manquantes depuis Binance
        let complete_task = Task::perform(
            async {
                // Attendre un peu pour que l'UI soit prête
                tokio::time::sleep(Duration::from_millis(500)).await;
                Message::CompleteMissingData
            },
            |_| Message::CompleteMissingData,
        );

        // Ouvrir la fenêtre principale
        let (main_id, open_task) = window::open(window::Settings {
            size: Size::new(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT),
            ..Default::default()
        });

        (
            Self { 
                chart_state, 
                tools_state, 
                settings_state: SettingsState::default(),
                chart_style,
                windows: WindowManager::new(main_id),
                editing_style: None,
                editing_color_index: None,
                binance_provider,
                realtime_enabled: true, // Activer le mode temps réel par défaut
                render_version: 0,
            },
            Task::batch(vec![
                open_task.map(Message::MainWindowOpened),
                complete_task,
            ]),
        )
    }

    fn title(&self, window_id: window::Id) -> String {
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => String::from("Settings - Style Chart"),
            Some(WindowType::Main) | None => {
                // Afficher le symbole de la série active, ou un titre par défaut
                if let Some(active_series) = self.chart_state.series_manager.active_series().next() {
                    active_series.symbol.clone()
                } else {
                    String::from("CandleChart")
                }
            }
        }
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.realtime_enabled {
            // Subscription pour les mises à jour en temps réel
            Subscription::batch(vec![
                iced::time::every(Duration::from_secs_f64(REALTIME_UPDATE_INTERVAL_SECS))
                    .map(|_| Message::RealtimeUpdate),
                window::close_events().map(Message::WindowClosed),
            ])
        } else {
            window::close_events().map(Message::WindowClosed)
        }
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
                if self.windows.is_open(WindowType::Settings) {
                    return Task::none();
                }
                self.editing_style = Some(self.chart_style.clone());
                self.editing_color_index = None;
                
                let (id, task) = window::open(window::Settings {
                    size: Size::new(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT),
                    resizable: false,
                    ..Default::default()
                });
                self.windows.set_id(WindowType::Settings, id);
                task.map(Message::SettingsWindowOpened)
            }
            
            Message::SettingsWindowOpened(_id) => Task::none(),
            
            Message::WindowClosed(id) => {
                match self.windows.get_window_type(id) {
                    Some(WindowType::Settings) => {
                        self.windows.remove_id(WindowType::Settings);
                        self.editing_style = None;
                        self.editing_color_index = None;
                    }
                    Some(WindowType::Main) => {
                        self.windows.remove_id(WindowType::Main);
                        // Fermer la fenêtre settings si elle est ouverte
                        if let Some(settings_id) = self.windows.get_id(WindowType::Settings) {
                            return window::close(settings_id);
                        }
                    }
                    None => {}
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
                if let Some(id) = self.windows.get_id(WindowType::Settings) {
                    self.windows.remove_id(WindowType::Settings);
                    self.editing_color_index = None;
                    return window::close(id);
                }
                Task::none()
            }
            
            Message::CancelSettings => {
                self.editing_style = None;
                self.editing_color_index = None;
                if let Some(id) = self.windows.get_id(WindowType::Settings) {
                    self.windows.remove_id(WindowType::Settings);
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
            
            Message::ToggleAutoScroll => {
                if let Some(ref mut style) = self.editing_style {
                    style.auto_scroll_enabled = !style.auto_scroll_enabled;
                }
                Task::none()
            }
            
            // === Messages temps réel ===
            Message::CompleteMissingData => {
                self.complete_missing_data();
                Task::none()
            }
            
            Message::RealtimeUpdate => {
                self.update_realtime()
            }
            
            Message::RealtimeUpdateComplete(results) => {
                println!("📥 RealtimeUpdateComplete: {} résultats reçus", results.len());
                self.apply_realtime_updates(results);
                Task::none()
            }
        }
    }
    
    /// Complète les données manquantes depuis Binance pour toutes les séries
    fn complete_missing_data(&mut self) {
        println!("🔄 Complétion des données manquantes depuis Binance...");
        
        // Collecter toutes les informations nécessaires d'abord
        let mut updates: Vec<(SeriesId, String, Option<i64>)> = Vec::new();
        
        for series in self.chart_state.series_manager.all_series() {
            let series_id = series.id.clone();
            let series_name = series.full_name();
            
            // Vérifier si le format est compatible avec Binance (SYMBOL_INTERVAL)
            if !series_name.contains('_') {
                println!("  ⚠️  {}: Format incompatible avec Binance (attendu: SYMBOL_INTERVAL)", series_name);
                continue;
            }
            
            // Récupérer le dernier timestamp connu
            let last_ts = series.data.max_timestamp();
            updates.push((series_id, series_name, last_ts));
        }
        
        // Maintenant faire les mises à jour
        for (series_id, series_name, last_ts) in updates {
            
            if let Some(last_timestamp) = last_ts {
                // Calculer le timestamp actuel
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                // Extraire l'intervalle depuis le nom de la série (format: SYMBOL_INTERVAL)
                let interval = series_name.split('_').last().unwrap_or("1h");
                
                // Calculer le seuil pour déterminer si les données sont récentes (2 intervalles)
                let threshold_seconds = calculate_candles_back_timestamp(interval, 2);
                
                // Si les données sont récentes (moins de 2 intervalles), on complète
                // Sinon, on récupère depuis le dernier timestamp
                let since_ts = if now - last_timestamp < threshold_seconds {
                    last_timestamp
                } else {
                    // Si les données sont anciennes, on récupère les 100 dernières bougies
                    println!("  ℹ️  {}: Données anciennes, récupération des 100 dernières bougies", series_name);
                    // Calculer dynamiquement selon l'intervalle
                    now - calculate_candles_back_timestamp(interval, 100)
                };
                
                println!("  📥 {}: Récupération depuis le timestamp {}", series_name, since_ts);
                
                match self.chart_state.fetch_new_candles_from_provider(
                    &series_id,
                    since_ts,
                    &self.binance_provider,
                ) {
                    UpdateResult::MultipleCandlesAdded(n) => {
                        println!("  ✅ {}: {} nouvelles bougies ajoutées", series_name, n);
                    }
                    UpdateResult::NoUpdate => {
                        println!("  ℹ️  {}: Aucune nouvelle bougie", series_name);
                    }
                    UpdateResult::Error(e) => {
                        println!("  ❌ {}: Erreur - {}", series_name, e);
                    }
                    _ => {}
                }
            } else {
                // Aucune donnée, synchroniser complètement
                println!("  📥 {}: Aucune donnée, synchronisation complète", series_name);
                match self.chart_state.sync_from_provider(&series_id, &self.binance_provider) {
                    UpdateResult::MultipleCandlesAdded(n) => {
                        println!("  ✅ {}: {} bougies synchronisées", series_name, n);
                    }
                    UpdateResult::Error(e) => {
                        println!("  ❌ {}: Erreur - {}", series_name, e);
                    }
                    _ => {}
                }
            }
        }
        
        // Ajuster le viewport une seule fois à la fin (si auto-scroll activé)
        if self.chart_style.auto_scroll_enabled {
            self.chart_state.auto_scroll_to_latest();
        }
        println!("✅ Complétion terminée");
    }
    
    /// Met à jour les données en temps réel pour les séries actives
    /// 
    /// Utilise Iced Tasks pour faire les requêtes en parallèle sans bloquer le thread principal.
    fn update_realtime(&mut self) -> Task<Message> {
        if !self.realtime_enabled {
            return Task::none();
        }
        
        // Collecter les IDs des séries actives d'abord
        let active_series: Vec<(SeriesId, String)> = self.chart_state.series_manager
            .active_series()
            .filter_map(|s| {
                let name = s.full_name();
                // Vérifier si le format est compatible avec Binance
                if name.contains('_') {
                    Some((s.id.clone(), name))
                } else {
                    None
                }
            })
            .collect();
        
        if active_series.is_empty() {
            return Task::none();
        }
        
        // Cloner le provider pour l'utiliser dans la Task async
        let provider = self.binance_provider.clone();
        
        // Créer une Task async qui fait toutes les requêtes en parallèle
        println!("🚀 Démarrage des requêtes async pour {} série(s)", active_series.len());
        Task::perform(
            async move {
                use futures::future::join_all;
                
                // Créer un vecteur de futures pour toutes les requêtes
                let futures: Vec<_> = active_series
                    .iter()
                    .map(|(series_id, series_name)| {
                        let provider = provider.clone();
                        let series_id = series_id.clone();
                        let series_name = series_name.clone();
                        
                        async move {
                            let result = provider.get_latest_candle_async(&series_id).await;
                            (series_id, series_name, result)
                        }
                    })
                    .collect();
                
                // Exécuter toutes les requêtes en parallèle
                let results = join_all(futures).await;
                println!("✅ Toutes les requêtes async terminées");
                results
            },
            Message::RealtimeUpdateComplete,
        )
    }
    
    /// Applique les résultats des mises à jour en temps réel
    fn apply_realtime_updates(&mut self, results: Vec<(SeriesId, String, Result<Option<Candle>, String>)>) {
        let mut has_updates = false;
        let mut has_new_candles = false;
        
        for (series_id, series_name, result) in results {
            match result {
                Ok(Some(candle)) => {
                    match self.chart_state.update_candle(&series_id, candle) {
                        UpdateResult::NewCandle => {
                            println!("🔄 {}: Nouvelle bougie ajoutée", series_name);
                            has_updates = true;
                            has_new_candles = true;
                        }
                        UpdateResult::CandleUpdated => {
                            // Bougie mise à jour - on marque aussi comme update pour le re-render
                            has_updates = true;
                        }
                        UpdateResult::Error(e) => {
                            eprintln!("❌ {}: Erreur mise à jour - {}", series_name, e);
                        }
                        _ => {}
                    }
                }
                Ok(None) => {
                    // Aucune nouvelle bougie
                }
                Err(e) => {
                    eprintln!("❌ {}: Erreur récupération - {}", series_name, e);
                }
            }
        }
        
        // Ajuster le viewport si nécessaire (si auto-scroll activé et nouvelles bougies)
        if has_new_candles && self.chart_style.auto_scroll_enabled {
            self.chart_state.auto_scroll_to_latest();
        }
        
        // Forcer le re-render en incrémentant le compteur de version
        // Cela permet à Iced de détecter que l'état a changé et de re-rendre le canvas
        if has_updates {
            self.render_version = self.render_version.wrapping_add(1);
        }
    }

    /// Helper pour finaliser l'édition d'un rectangle avec historique
    fn finish_rectangle_edit(&mut self) {
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
    
    /// Helper pour finaliser l'édition d'une ligne horizontale avec historique
    fn finish_hline_edit(&mut self) {
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
    
    /// Helper pour supprimer un élément sélectionné avec historique
    fn delete_selected(&mut self) {
        // Supprimer rectangle sélectionné
        if let Some(index) = self.tools_state.editing.selected_index {
            if index < self.tools_state.rectangles.len() {
                let deleted_rect = self.tools_state.rectangles[index].clone();
                self.tools_state.history.record(HistoryAction::DeleteRectangle { 
                    index, 
                    rect: deleted_rect 
                });
                self.tools_state.rectangles.remove(index);
                self.tools_state.editing.deselect();
                return;
            }
        }
        
        // Supprimer ligne horizontale sélectionnée
        if let Some(index) = self.tools_state.hline_editing.selected_index {
            if index < self.tools_state.horizontal_lines.len() {
                let deleted_line = self.tools_state.horizontal_lines[index].clone();
                self.tools_state.history.record(HistoryAction::DeleteHLine { 
                    index, 
                    line: deleted_line 
                });
                self.tools_state.horizontal_lines.remove(index);
                self.tools_state.hline_editing.deselect();
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
                self.finish_rectangle_edit();
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
                self.finish_hline_edit();
            }
            ChartMessage::DeselectHLine => {
                self.tools_state.hline_editing.deselect();
            }
            
            // === Suppression ===
            ChartMessage::DeleteSelected => {
                self.delete_selected();
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
        match self.windows.get_window_type(window_id) {
            Some(WindowType::Settings) => self.view_settings(),
            Some(WindowType::Main) | None => self.view_main(),
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

        // Toggle pour l'auto-scroll
        let auto_scroll_enabled = editing_style
            .map(|s| s.auto_scroll_enabled)
            .unwrap_or(true);
        
        let auto_scroll_toggle = row![
            checkbox(auto_scroll_enabled)
                .on_toggle(|_| Message::ToggleAutoScroll),
            text("Défilement automatique vers les dernières données")
                .size(14)
                .color(Color::from_rgb(0.8, 0.8, 0.8))
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

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
            auto_scroll_toggle,
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
