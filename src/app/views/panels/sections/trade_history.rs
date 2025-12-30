//! Section "Historique des trades"

use iced::widget::{scrollable, text, column, row, Space, container};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::colors,
    data::TradeType,
};

/// Vue pour la section "Historique des trades"
pub fn view_trade_history(app: &ChartApp) -> Element<'_, Message> {
    let trade_history = &app.trading_state.trade_history;
    let trades = &trade_history.trades;
    
    // Récupérer le prix actuel pour calculer le P&L non réalisé
    let current_price = app.chart_state.series_manager
        .active_series()
        .next()
        .and_then(|s| s.data.last_candle().map(|c| c.close))
        .unwrap_or(0.0);
    
    let current_symbol = app.chart_state.series_manager
        .active_series()
        .next()
        .map(|s| s.symbol.as_str())
        .unwrap_or("");
    
    // Style pour les cartes
    let card_style = move |_theme: &iced::Theme| {
        container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.15))),
            border: iced::Border {
                color: Color::from_rgb(0.2, 0.2, 0.25),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    };
    
    // Calculer le P&L total réalisé et non réalisé
    let total_realized_pnl = trade_history.total_realized_pnl();
    let total_unrealized_pnl = if current_price > 0.0 && !current_symbol.is_empty() {
        trade_history.total_unrealized_pnl(current_symbol, current_price)
    } else {
        0.0
    };
    let total_pnl = total_realized_pnl + total_unrealized_pnl;
    
    let initial_balance = 10000.0; // Solde initial par défaut
    let pnl_percentage = if initial_balance > 0.0 {
        (total_pnl / initial_balance) * 100.0
    } else {
        0.0
    };
    
    let pnl_color = if total_pnl >= 0.0 {
        Color::from_rgb(0.0, 0.8, 0.0)
    } else {
        Color::from_rgb(0.8, 0.0, 0.0)
    };
    
    // En-tête avec P&L total et pourcentage
    let header = container(
        column![
            row![
                text("Historique des Trades")
                    .size(20)
                    .color(colors::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill),
            if !trades.is_empty() || !trade_history.open_positions.is_empty() {
                column![
                    row![
                        text(format!("P&L Total: {:.2} USDT", total_pnl))
                            .size(14)
                            .color(pnl_color)
                            .font(iced::Font::MONOSPACE),
                        Space::new().width(Length::Fill),
                        text(format!("({:+.2}%)", pnl_percentage))
                            .size(14)
                            .color(pnl_color)
                            .font(iced::Font::MONOSPACE),
                    ]
                    .width(Length::Fill),
                    if total_unrealized_pnl != 0.0 {
                        row![
                            text(format!("  Réalisé: {:.2} USDT", total_realized_pnl))
                                .size(12)
                                .color(colors::TEXT_SECONDARY)
                                .font(iced::Font::MONOSPACE),
                            Space::new().width(Length::Fixed(16.0)),
                            text(format!("Non réalisé: {:.2} USDT", total_unrealized_pnl))
                                .size(12)
                                .color(if total_unrealized_pnl >= 0.0 {
                                    Color::from_rgb(0.0, 0.8, 0.0)
                                } else {
                                    Color::from_rgb(0.8, 0.0, 0.0)
                                })
                                .font(iced::Font::MONOSPACE),
                        ]
                        .width(Length::Fill)
                    } else {
                        row![].width(Length::Fill)
                    },
                ]
                .spacing(4)
                .width(Length::Fill)
            } else {
                column![].width(Length::Fill)
            },
        ]
        .spacing(8)
    )
    .padding(12)
    .style(card_style)
    .width(Length::Fill);
    
    // Liste des trades (du plus récent au plus ancien)
    let mut trades_list = column![].spacing(8);
    
    if trades.is_empty() {
        trades_list = trades_list.push(
            container(
                text("Aucun trade effectué")
                    .size(14)
                    .color(colors::TEXT_SECONDARY)
            )
            .padding(20)
            .width(Length::Fill)
            .center_x(Length::Fill)
        );
    } else {
        // Afficher les trades du plus récent au plus ancien
        for trade in trades.iter().rev() {
            let trade_type_text = match trade.trade_type {
                TradeType::Buy => "ACHAT",
                TradeType::Sell => "VENTE",
            };

            let trade_type_color = match trade.trade_type {
                TradeType::Buy => Color::from_rgb(0.0, 0.8, 0.0),
                TradeType::Sell => Color::from_rgb(0.8, 0.0, 0.0),
            };
            
            let pnl_color = if trade.realized_pnl >= 0.0 {
                Color::from_rgb(0.0, 0.8, 0.0)
            } else {
                Color::from_rgb(0.8, 0.0, 0.0)
            };
            
            // Formater le timestamp
            let datetime = chrono::DateTime::from_timestamp(trade.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| String::from("N/A"));
            
            let trade_card = container(
                column![
                    row![
                        text(format!("Trade #{}", trade.id))
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text(trade_type_text)
                            .size(12)
                            .color(trade_type_color)
                            .font(iced::Font::MONOSPACE),
                    ]
                    .width(Length::Fill),
                    Space::new().height(Length::Fixed(8.0)),
                    row![
                        text(format!("{}", trade.symbol))
                            .size(14)
                            .color(colors::TEXT_PRIMARY)
                            .font(iced::Font::MONOSPACE),
                        Space::new().width(Length::Fill),
                        text(format!("{:.6}", trade.quantity))
                            .size(12)
                            .color(colors::TEXT_SECONDARY)
                            .font(iced::Font::MONOSPACE),
                    ]
                    .width(Length::Fill),
                    Space::new().height(Length::Fixed(4.0)),
                    row![
                        text(format!("Prix: {:.2} USDT", trade.price))
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text(format!("Total: {:.2} USDT", trade.total_amount))
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                    ]
                    .width(Length::Fill),
                    if trade.realized_pnl != 0.0 {
                        row![
                            text("P&L:")
                                .size(12)
                                .color(colors::TEXT_SECONDARY),
                            Space::new().width(Length::Fixed(8.0)),
                            text(format!("{:.2} USDT", trade.realized_pnl))
                                .size(12)
                                .color(pnl_color)
                                .font(iced::Font::MONOSPACE),
                        ]
                        .width(Length::Fill)
                    } else {
                        row![].width(Length::Fill)
                    },
                    Space::new().height(Length::Fixed(4.0)),
                    text(datetime)
                        .size(10)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(4)
            )
            .padding(12)
            .style(card_style)
            .width(Length::Fill);
            
            trades_list = trades_list.push(trade_card);
        }
    }
    
    container(
        scrollable(
            column![
                header,
                Space::new().height(Length::Fixed(16.0)),
                trades_list,
            ]
            .spacing(0)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(colors::BACKGROUND_MEDIUM)),
        ..Default::default()
    })
    .into()
}



