//! Vue principale de l'application

use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Element, Length};
use crate::finance_chart::{
    chart, x_axis, y_axis, tools_panel, series_select_box,
    X_AXIS_HEIGHT, TOOLS_PANEL_WIDTH,
};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::{self, colors},
    drag_overlay,
};
use super::helpers::corner_settings_button;
use super::indicators::chart_with_indicators_overlay;
use super::panels::{view_right_panel, view_bottom_panel, build_indicator_panels};

/// Composant qui regroupe toutes les sections du graphique
fn view_chart_component(app: &ChartApp) -> Element<'_, Message> {
    let panel_focused = app.panels.has_focused_panel();
    
    // Ligne principale : Tools (gauche) + Chart (centre) + Axe Y (droite)
    let chart_row = row![
        tools_panel(&app.tools_state, app.indicators_panel_open).map(Message::ToolsPanel),
        mouse_area(
            container(
                chart(&app.chart_state, &app.tools_state, &app.settings_state, &app.chart_style, panel_focused)
                    .map(Message::Chart)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_enter(Message::ClearPanelFocus),
        y_axis(&app.chart_state).map(Message::YAxis)
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // Ligne du bas : espace comblé (sous tools) + Axe X + bouton settings (coin)
    let x_axis_row = row![
        container(Space::new())
            .width(Length::Fixed(TOOLS_PANEL_WIDTH))
            .height(Length::Fill)
            .style(view_styles::dark_background_style),
        x_axis(&app.chart_state).map(Message::XAxis),
        corner_settings_button()
    ]
    .width(Length::Fill)
    .height(Length::Fixed(X_AXIS_HEIGHT));

    // Construire le layout avec les indicateurs
    let mut layout_items: Vec<Element<'_, Message>> = vec![chart_row.into()];
    
    // Ajouter les panneaux d'indicateurs (Volume, RSI, MACD)
    build_indicator_panels(app, &mut layout_items);
    
    // Ajouter l'axe X en dernier
    layout_items.push(x_axis_row.into());
    
    // Créer le layout principal
    let mut main_chart_layout = column![];
    for item in layout_items {
        main_chart_layout = main_chart_layout.push(item);
    }
    let main_chart_layout = main_chart_layout
        .width(Length::Fill)
        .height(Length::Fill);
    
    // Ajouter l'overlay d'indicateurs si ouvert
    chart_with_indicators_overlay(main_chart_layout.into(), app)
}

/// Vue principale de l'application avec gestion globale du drag & drop
pub fn view_main(app: &ChartApp) -> Element<'_, Message> {
    // Récupérer le symbole de la série active pour le titre
    let title_text = app.chart_state.series_manager
        .active_series()
        .next()
        .map(|series| series.symbol.clone())
        .unwrap_or_else(|| String::from("Chart Candlestick"));
    
    // Header avec titre, bouton de téléchargements, bouton de configuration et select box de séries
    // Afficher le nombre de téléchargements en cours
    let downloads_count = app.download_manager.count();
    
    // Récupérer les infos de la série active pour le label de complétude
    let series_status = app.chart_state.series_manager
        .active_series()
        .next()
        .map(|series| {
            let candle_count = series.data.len();
            let oldest = series.data.min_timestamp();
            let newest = series.data.max_timestamp();
            
            // Vérifier s'il y a des gaps (sans appel API)
            let has_gaps = crate::app::realtime::has_gaps_to_fill(app, &series.id);
            
            // La série est complète si elle a des bougies et pas de gaps
            let is_complete = candle_count > 0 && !has_gaps;
            
            (is_complete, candle_count, oldest, newest)
        });
    
    let status_label = if let Some((is_complete, count, oldest, newest)) = series_status {
        if is_complete {
            let range_text = if let (Some(old), Some(new)) = (oldest, newest) {
                let duration_days = (new - old) / 86400;
                format!("✅ Sans gaps détectés ({} bougies, {} jours)", count, duration_days)
            } else {
                format!("✅ Sans gaps détectés ({} bougies)", count)
            };
            text(range_text)
                .size(12)
                .color(iced::Color::from_rgb(0.2, 0.8, 0.2))
        } else {
            // Distinguer entre série vide et série avec gaps
            if count == 0 {
                text("⚠️ Série vide")
                    .size(12)
                    .color(iced::Color::from_rgb(1.0, 0.7, 0.0))
            } else {
                text(format!("⚠️ Gaps détectés ({} bougies)", count))
                    .size(12)
                    .color(iced::Color::from_rgb(1.0, 0.7, 0.0))
            }
        }
    } else {
        text("")
            .size(12)
    };
    
    // Pas de boutons d'action pour l'instant
    let action_buttons = row![];
    
    let header_row = row![
        text(title_text)
            .size(24)
            .color(colors::TEXT_PRIMARY),
        Space::new().width(Length::Fixed(20.0)),
        if downloads_count > 0 {
            text(format!("📥 {} téléchargement(s)", downloads_count))
                .size(14)
                .color(iced::Color::from_rgb(0.2, 0.6, 1.0))
        } else {
            text("")
        },
        Space::new().width(Length::Fill),
        button("📥 Téléchargements")
            .on_press(Message::OpenDownloads)
            .style(view_styles::icon_button_style),
        Space::new().width(Length::Fixed(10.0)),
        button("⚙️ Provider")
            .on_press(Message::OpenProviderConfig)
            .style(view_styles::icon_button_style),
        Space::new().width(Length::Fixed(10.0)),
        action_buttons,
        Space::new().width(Length::Fixed(10.0)),
        series_select_box(&app.chart_state.series_manager).map(Message::SeriesPanel),
        Space::new().width(Length::Fixed(10.0)),
        status_label
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);
    
    let header = container(header_row)
    .width(Length::Fill)
    .padding(15)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(colors::BACKGROUND_HEADER)),
        ..Default::default()
    });

    // Zone principale : Composant chart + Panneau de droite (si visible)
    let main_content = if app.panels.right.visible {
        row![
            view_chart_component(app),
            view_right_panel(app)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        row![view_chart_component(app)]
            .width(Length::Fill)
            .height(Length::Fill)
    };

    // Layout complet : Header + Zone principale + Panneau du bas
    let main_layout = column![
        header,
        main_content,
        view_bottom_panel(app)
    ]
    .width(Length::Fill)
    .height(Length::Fill);
    
    // Si on drag une section, ajouter l'overlay qui suit la souris
    if let Some(section) = app.dragging_section {
        let position = app.drag_position.unwrap_or(iced::Point::new(0.0, 0.0));
        
        container(
            column![
                main_layout,
                drag_overlay::drag_overlay(section, position)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        main_layout.into()
    }
}

