//! Vue principale de l'application

use iced::widget::{button, column, container, mouse_area, row, stack, text, Space};
use iced::{Element, Length};
use crate::finance_chart::{
    chart, chart_with_trading, chart_with_trades_and_trading,
    x_axis, y_axis, tools_panel, series_select_box,
    X_AXIS_HEIGHT, TOOLS_PANEL_WIDTH,
};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::{self, colors},
};
use super::helpers::corner_settings_button;
use super::indicators::chart_with_indicators_overlay;
use super::panels::{view_right_panel, view_bottom_panel, build_indicator_panels, section_context_menu};
use super::crosshair_overlay::crosshair_overlay;

/// Composant qui regroupe toutes les sections du graphique
fn view_chart_component(app: &ChartApp) -> Element<'_, Message> {
    let panel_focused = app.ui.panels.has_focused_panel();

    // Créer le graphique principal (sans la tools bar qui sera en overlay)
    let main_chart = mouse_area(
        container({
            // Récupérer le symbole actuel et les trades
            let current_symbol = app.chart_state.series_manager
                .active_series()
                .next()
                .map(|s| s.symbol.as_str())
                .unwrap_or("");

            let trades = &app.trading_state.trade_history.trades;

            // Utiliser chart_with_trades_and_trading si on est en mode paper et qu'il y a des trades
            // Sinon utiliser chart_with_trading pour afficher les ordres limit même sans trades
            let backtest_state = Some(&app.ui.backtest_state);
            if app.account_type.is_demo() && !current_symbol.is_empty() {
                if !trades.is_empty() {
                    chart_with_trades_and_trading(
                        &app.chart_state,
                        &app.tools_state,
                        &app.settings_state,
                        &app.chart_style,
                        panel_focused,
                        trades,
                        current_symbol,
                        &app.trading_state,
                        app.indicators.bollinger_bands_enabled,
                        app.indicators.moving_average_enabled,
                        Some(&app.indicators.params),
                        backtest_state,
                    )
                    .map(Message::Chart)
                } else {
                    chart_with_trading(
                        &app.chart_state,
                        &app.tools_state,
                        &app.settings_state,
                        &app.chart_style,
                        panel_focused,
                        &app.trading_state,
                        current_symbol,
                        app.indicators.bollinger_bands_enabled,
                        app.indicators.moving_average_enabled,
                        Some(&app.indicators.params),
                        backtest_state,
                    )
                    .map(Message::Chart)
                }
            } else {
                chart(&app.chart_state, &app.tools_state, &app.settings_state, &app.chart_style, panel_focused, app.indicators.bollinger_bands_enabled, app.indicators.moving_average_enabled, Some(&app.indicators.params), backtest_state)
                    .map(Message::Chart)
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .on_enter(Message::ClearPanelFocus);

    // Axe Y à droite
    let y_axis_element = y_axis(&app.chart_state).map(Message::YAxis);

    // Ligne principale du graphique : Chart (gauche) + Axe Y (droite)
    let chart_area = row![
        main_chart,
        y_axis_element
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    // Axe X en bas
    let x_axis_element = x_axis(&app.chart_state).map(Message::XAxis);

    // Ligne du bas : Axe X + bouton settings (coin)
    let bottom_row = row![
        x_axis_element,
        corner_settings_button()
    ]
    .width(Length::Fill)
    .height(Length::Fixed(X_AXIS_HEIGHT));

    // Construire le layout vertical avec les indicateurs
    let mut layout_items: Vec<Element<'_, Message>> = vec![chart_area.into()];

    // Ajouter les panneaux d'indicateurs (Volume, RSI, MACD)
    build_indicator_panels(app, &mut layout_items);

    // Ajouter l'axe X en bas
    layout_items.push(bottom_row.into());

    // Créer le layout principal vertical
    let mut main_chart_layout = column![];
    for item in layout_items {
        main_chart_layout = main_chart_layout.push(item);
    }
    let main_chart_layout = main_chart_layout
        .width(Length::Fill)
        .height(Length::Fill);

    // Créer la tools bar en overlay qui fait toute la hauteur
    let tools_overlay = container(
        tools_panel(&app.tools_state, app.ui.indicators_panel_open).map(Message::ToolsPanel)
    )
    .width(Length::Fixed(TOOLS_PANEL_WIDTH))
    .height(Length::Fill)
    .style(view_styles::dark_background_style);

    // Superposer la tools bar sur le layout principal
    let layout_stack = stack![
        main_chart_layout,
        container(tools_overlay)
            .width(Length::Fixed(TOOLS_PANEL_WIDTH))
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top),
        // Overlay pour la barre verticale synchronisée du crosshair
        container(crosshair_overlay(&app.chart_state))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: None,
                ..Default::default()
            }),
        // Overlay pour la barre verticale de sélection du backtest
        container(super::backtest_overlay::backtest_overlay(&app.chart_state, &app.ui.backtest_state))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: None,
                ..Default::default()
            })
    ];

    // Ajouter l'overlay d'indicateurs si ouvert
    chart_with_indicators_overlay(layout_stack.into(), app)
}

/// Vue principale de l'application
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
    
    // Créer le select d'actifs si des actifs sont sélectionnés
    let asset_select: Element<'_, Message> = if !app.selected_assets.is_empty() {
        let mut selected_assets_vec: Vec<String> = app.selected_assets.iter().cloned().collect();
        selected_assets_vec.sort(); // Trier pour un affichage cohérent
        
        // Utiliser le symbole mémorisé depuis le pick_list
        // Si aucun symbole n'est mémorisé, utiliser le premier actif sélectionné
        // NE PAS utiliser le symbole de la série active comme fallback pour éviter qu'il change lors des changements de série
        let current_symbol = app.selected_asset_symbol
            .clone()
            .filter(|sym| selected_assets_vec.contains(sym))
            .or_else(|| selected_assets_vec.first().cloned());
        
        let selected_value = current_symbol;
        
        use iced::widget::pick_list;
        pick_list(
            selected_assets_vec,
            selected_value,
            move |selected: String| {
                Message::SelectAssetFromHeader(selected)
            }
        )
        .width(Length::Fixed(200.0))
        .text_size(18.0)
        .into()
    } else {
        text(title_text)
            .size(24)
            .color(colors::TEXT_PRIMARY)
            .into()
    };
    
    let header_row = row![
        asset_select,
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
        button("💰 Actifs")
            .on_press(Message::OpenAssets)
            .style(view_styles::icon_button_style),
        Space::new().width(Length::Fixed(10.0)),
        action_buttons,
        Space::new().width(Length::Fixed(10.0)),
        series_select_box(&app.chart_state.series_manager, app.selected_asset_symbol.as_ref()).map(Message::SeriesPanel),
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
    let main_content = if app.ui.panels.right.visible {
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

    // Menu contextuel des sections en overlay positionné à la position du clic
    let section_context_menu_overlay = if let Some((_, position)) = &app.ui.section_context_menu {
        stack![
            // Overlay sombre pour fermer le menu en cliquant ailleurs
            mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .on_press(Message::CloseSectionContextMenu),
            // Menu contextuel positionné à la position du clic
            container(
                section_context_menu(app)
            )
            .style(|_theme| container::Style {
                background: None,
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .padding(iced::Padding {
                left: position.x,
                top: position.y,
                right: 0.0,
                bottom: 0.0,
            })
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        stack![].width(Length::Fill).height(Length::Fill)
    };
    
    // Menu contextuel du graphique en overlay positionné à la position du clic
    let chart_context_menu_overlay = if let Some(position) = &app.ui.chart_context_menu {
        stack![
            // Overlay sombre pour fermer le menu en cliquant ailleurs
            mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .on_press(Message::CloseChartContextMenu),
            // Menu contextuel positionné à la position du clic
            container(
                chart_context_menu()
            )
            .style(|_theme| container::Style {
                background: None,
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Top)
            .padding(iced::Padding {
                left: position.x,
                top: position.y,
                right: 0.0,
                bottom: 0.0,
            })
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        stack![].width(Length::Fill).height(Length::Fill)
    };
    
    // Layout complet : Header + Zone principale + Panneau du bas + Menus contextuels
    stack![
        column![
            header,
            main_content,
            view_bottom_panel(app)
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        section_context_menu_overlay,
        chart_context_menu_overlay
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Menu contextuel du graphique principal
fn chart_context_menu() -> Element<'static, Message> {
    use iced::widget::{button, column, container};
    use iced::{Length, Color};
    use crate::app::view_styles;
    
    let menu_items = column![
        button("🔄 Reset View")
            .on_press(Message::ResetView)
            .style(view_styles::icon_button_style)
            .width(Length::Fill),
    ]
    .spacing(4)
    .padding(8);
    
    container(menu_items)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.18))),
            border: iced::Border {
                color: Color::from_rgb(0.3, 0.3, 0.35),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fixed(150.0))
        .into()
}


