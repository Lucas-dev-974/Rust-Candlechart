//! Section "Backtest"

use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text};
use iced::{Element, Length, Color};
use crate::app::{app_state::ChartApp, messages::Message, view_styles::colors};

/// Style pour les boutons primaires
fn primary_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.8))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.3, 0.6, 0.9),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Style pour les boutons secondaires
fn secondary_button_style(_theme: &iced::Theme, _status: iced::widget::button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.4))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.4, 0.4, 0.5),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// Formate un timestamp en date lisible
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc, TimeZone};
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    datetime.format("%d/%m/%Y %H:%M:%S").to_string()
}

/// Vue pour la section "Backtest"
pub fn view_backtest(app: &ChartApp) -> Element<'_, Message> {
    let backtest_state = &app.ui.backtest_state;
    let _has_start_date = backtest_state.start_timestamp.is_some();
    let is_playing = backtest_state.is_playing;
    let is_enabled = backtest_state.enabled;
    
    let mut content = column![]
        .spacing(15)
        .padding(20);
    
    // Titre
    content = content.push(
        text("Backtest")
            .size(18)
            .color(colors::TEXT_PRIMARY)
    );
    
    // Case à cocher "Activé backtest"
    content = content.push(
        row![
            checkbox(is_enabled)
                .on_toggle(|_| Message::ToggleBacktestEnabled),
            text("Activé backtest")
                .size(14)
                .color(colors::TEXT_PRIMARY)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    );
    
    // Informations sur la date sélectionnée (seulement si activé)
    if is_enabled {
        // Liste des stratégies disponibles
        let strategies = app.strategy_manager.get_all();
        
        // Créer une liste de tuples (ID, nom affiché) pour faciliter la sélection
        let strategy_list: Vec<(String, String)> = strategies
            .iter()
            .map(|(id, reg)| {
                (id.clone(), format!("{} ({})", reg.strategy.name(), id))
            })
            .collect();
        
        // Créer la liste des options affichées (avec "Aucune stratégie" en premier)
        let mut all_options = vec!["Aucune stratégie".to_string()];
        all_options.extend(strategy_list.iter().map(|(_, display)| display.clone()));
        
        // Trouver l'option sélectionnée
        let selected_option = if let Some(ref selected_id) = backtest_state.selected_strategy_id {
            strategy_list
                .iter()
                .find(|(id, _)| id == selected_id)
                .map(|(_, display)| display.clone())
        } else {
            Some("Aucune stratégie".to_string())
        };
        
        // Sélecteur de stratégie
        content = content.push(
            column![
                text("Algorithme de trading:")
                    .size(12)
                    .color(colors::TEXT_SECONDARY),
                pick_list(
                    all_options,
                    selected_option,
                    move |selected: String| {
                        if selected == "Aucune stratégie" {
                            Message::SelectBacktestStrategy(None)
                        } else {
                            // Trouver l'ID correspondant au nom sélectionné
                            let strategy_id = strategy_list
                                .iter()
                                .find(|(_, display)| *display == selected)
                                .map(|(id, _)| id.clone());
                            Message::SelectBacktestStrategy(strategy_id)
                        }
                    }
                )
                .width(Length::Fill)
                .placeholder("Sélectionner un algorithme...")
                .text_size(13.0)
            ]
            .spacing(5)
        );
        if let Some(timestamp) = backtest_state.start_timestamp {
            content = content.push(
                column![
                    text("Date de départ sélectionnée:")
                        .size(12)
                        .color(colors::TEXT_SECONDARY),
                    text(format_timestamp(timestamp))
                        .size(14)
                        .color(colors::TEXT_PRIMARY)
                ]
                .spacing(5)
            );
        } else {
            content = content.push(
                text("Cliquez sur le graphique pour sélectionner une date de départ")
                    .size(12)
                    .color(colors::TEXT_SECONDARY)
            );
        }
        
        // Boutons de contrôle (seulement si activé)
        let mut controls_row = row![].spacing(10);
        
        if is_playing {
            // Bouton pause
            controls_row = controls_row.push(
                button("⏸ Pause")
                    .on_press(Message::PauseBacktest)
                    .style(secondary_button_style)
            );
        } else {
            // Bouton play
            controls_row = controls_row.push(
                button("▶ Play")
                    .on_press(Message::StartBacktest)
                    .style(primary_button_style)
            );
        }
        
        // Bouton stop
        controls_row = controls_row.push(
            button("⏹ Stop")
                .on_press(Message::StopBacktest)
                .style(secondary_button_style)
        );
        
        content = content.push(controls_row);
        
        // État de la lecture
        if is_playing {
            content = content.push(
                text(format!("Lecture en cours... (Index: {})", backtest_state.current_index))
                    .size(12)
                    .color(Color::from_rgb(0.2, 0.8, 0.3))
            );
        }
        
        // Afficher les statistiques du backtest
        // Toujours afficher les statistiques si le backtest a été démarré au moins une fois
        if backtest_state.start_index.is_some() || !backtest_state.backtest_trade_history.trades.is_empty() {
            let symbol = app.chart_state.series_manager
                .active_series()
                .next()
                .map(|s| s.symbol.clone())
                .unwrap_or_else(|| String::from("UNKNOWN"));
            
            let current_price = app.chart_state.series_manager
                .active_series()
                .next()
                .and_then(|s| s.data.last_candle().map(|c| c.close))
                .unwrap_or(0.0);
            
            let stats = backtest_state.calculate_stats(&symbol, current_price);
            
            // Statistiques du backtest
            content = content.push(
                column![
                    text("Statistiques du Backtest")
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    row![
                        text("Capital initial: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{:.2} USDT", stats.initial_capital))
                            .size(12)
                            .color(colors::TEXT_PRIMARY)
                    ]
                    .spacing(5),
                    row![
                        text("Capital final: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{:.2} USDT", stats.final_capital))
                            .size(12)
                            .color(if stats.final_capital >= stats.initial_capital {
                                Color::from_rgb(0.2, 0.8, 0.3)
                            } else {
                                Color::from_rgb(0.8, 0.2, 0.2)
                            })
                    ]
                    .spacing(5),
                    row![
                        text("P&L réalisé: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{:.2} USDT", stats.total_realized_pnl))
                            .size(12)
                            .color(if stats.total_realized_pnl >= 0.0 {
                                Color::from_rgb(0.2, 0.8, 0.3)
                            } else {
                                Color::from_rgb(0.8, 0.2, 0.2)
                            })
                    ]
                    .spacing(5),
                    row![
                        text("P&L non réalisé: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{:.2} USDT", stats.total_unrealized_pnl))
                            .size(12)
                            .color(if stats.total_unrealized_pnl >= 0.0 {
                                Color::from_rgb(0.2, 0.8, 0.3)
                            } else {
                                Color::from_rgb(0.8, 0.2, 0.2)
                            })
                    ]
                    .spacing(5),
                    row![
                        text("P&L total: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{:.2} USDT ({:.2}%)", stats.total_pnl, stats.return_percentage))
                            .size(12)
                            .color(if stats.total_pnl >= 0.0 {
                                Color::from_rgb(0.2, 0.8, 0.3)
                            } else {
                                Color::from_rgb(0.8, 0.2, 0.2)
                            })
                    ]
                    .spacing(5),
                    row![
                        text("Trades: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{}", stats.total_trades))
                            .size(12)
                            .color(colors::TEXT_PRIMARY)
                    ]
                    .spacing(5),
                    row![
                        text("Positions ouvertes: ")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        text(format!("{}", stats.open_positions))
                            .size(12)
                            .color(colors::TEXT_PRIMARY)
                    ]
                    .spacing(5),
                ]
                .spacing(8)
                .padding(10)
            );
        }
        
        // Historique des trades du backtest
        let backtest_trades = &backtest_state.backtest_trade_history.trades;
        if !backtest_trades.is_empty() {
            content = content.push(
                column![
                    text("Historique des Trades")
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    // Afficher les 10 derniers trades
                    {
                        let mut trades_column = column![].spacing(3);
                        for trade in backtest_trades.iter().rev().take(10) {
                            let trade_type_text = match trade.trade_type {
                                crate::app::data::TradeType::Buy => "ACHAT",
                                crate::app::data::TradeType::Sell => "VENTE",
                            };
                            let trade_type_color = match trade.trade_type {
                                crate::app::data::TradeType::Buy => Color::from_rgb(0.2, 0.8, 0.3),
                                crate::app::data::TradeType::Sell => Color::from_rgb(0.8, 0.2, 0.2),
                            };
                            
                            trades_column = trades_column.push(
                                row![
                                    text(format!("{}", trade_type_text))
                                        .size(11)
                                        .color(trade_type_color),
                                    text(format!(" {} @ {:.2}", trade.quantity, trade.price))
                                        .size(11)
                                        .color(colors::TEXT_SECONDARY),
                                    text(format!(" P&L: {:.2}", trade.realized_pnl))
                                        .size(11)
                                        .color(if trade.realized_pnl >= 0.0 {
                                            Color::from_rgb(0.2, 0.8, 0.3)
                                        } else {
                                            Color::from_rgb(0.8, 0.2, 0.2)
                                        })
                                ]
                                .spacing(5)
                            );
                        }
                        trades_column
                    }
                    .spacing(3)
                ]
                .spacing(8)
                .padding(10)
            );
        }
    }
    
    container(
        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}


