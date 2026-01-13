//! Section "Ordres" avec interface de trading

use iced::widget::{button, checkbox, column, container, row, text, text_input, Space};
use iced::{Element, Length, Color};
use crate::app::{
    app_state::ChartApp,
    messages::Message,
    view_styles::colors,
    data::OrderType,
};
use crate::finance_chart::core::{SeriesData, Candle};

/// Formate un prix pour l'affichage avec une précision adaptative
/// - >= 10000 : 0 décimales (ex: 90000)
/// - >= 100 : 1 décimale (ex: 123.4)
/// - >= 1 : 2 décimales (ex: 12.34)
/// - < 1 : 4 décimales (ex: 0.1234)
fn format_price_for_display(price: f64) -> String {
    if price >= 10000.0 {
        format!("{:.0}", price)
    } else if price >= 100.0 {
        format!("{:.1}", price)
    } else if price >= 1.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    }
}

/// Vue pour la section "Ordres" avec interface de trading
pub fn view_orders(app: &ChartApp) -> Element<'_, Message> {
    // Utiliser le symbole mémorisé depuis le pick_list si disponible, sinon le symbole de la série active
    // Cela garantit que le panel "Ordres" affiche le même symbole que le pick_list
    let symbol = app.selected_asset_symbol
        .clone()
        .or_else(|| {
            app.chart_state.series_manager
                .active_series()
                .next()
                .map(|s| s.symbol.clone())
        })
        .unwrap_or_else(|| String::from("N/A"));
    
    // Récupérer la quote currency (devise de cotation) pour ce symbole
    // Chercher dans la liste des assets chargés, sinon essayer de parser le symbole
    let quote_currency = app.assets
        .iter()
        .find(|a| a.symbol == symbol)
        .map(|a| a.quote_asset.clone())
        .unwrap_or_else(|| {
            // Fallback: essayer de deviner depuis le symbole (pas fiable pour tous les cas)
            // Par exemple: BTCUSDT -> USDT, BTCJPY -> JPY
            // Mais certains symboles comme BTCUSD1 ne suivent pas ce pattern
            if symbol.ends_with("USDT") {
                "USDT".to_string()
            } else if symbol.ends_with("JPY") {
                "JPY".to_string()
            } else if symbol.ends_with("IDR") {
                "IDR".to_string()
            } else if symbol.ends_with("EUR") {
                "EUR".to_string()
            } else if symbol.ends_with("GBP") {
                "GBP".to_string()
            } else {
                // Par défaut, essayer d'extraire les 3-4 derniers caractères
                // ou utiliser USDT comme fallback
                if symbol.len() >= 3 {
                    symbol[symbol.len().saturating_sub(4)..].to_string()
                } else {
                    "USDT".to_string() // Fallback par défaut
                }
            }
        });
    
    // Récupérer le prix actuel (dernière bougie)
    // Pour obtenir le prix réel du marché, on doit trouver la bougie la plus récente
    // parmi toutes les séries du même symbole, pas seulement la série active.
    // Cela garantit que le prix ne change pas quand on change de timeframe.
    let current_price = {
        // Chercher toutes les séries avec le même symbole
        let all_series_for_symbol: Vec<_> = app.chart_state.series_manager
            .all_series()
            .filter(|s| s.symbol == symbol)
            .collect();
        
        // Trouver la bougie la plus récente parmi toutes ces séries
        let mut latest_candle: Option<(&SeriesData, &Candle)> = None;
        for series in all_series_for_symbol {
            if let Some(candle) = series.data.last_candle() {
                match latest_candle {
                    None => latest_candle = Some((series, candle)),
                    Some((_, latest)) => {
                        // Comparer les timestamps pour trouver la plus récente
                        if candle.timestamp > latest.timestamp {
                            latest_candle = Some((series, candle));
                        }
                    }
                }
            }
        }
        
        // Utiliser le prix de la bougie la plus récente, ou fallback sur la série active
        latest_candle
            .map(|(_, candle)| candle.close)
            .or_else(|| {
                app.chart_state
                    .last_candle()
                    .map(|c| c.close)
            })
            .unwrap_or(0.0)
    };
    
    // Récupérer la quantité
    let quantity = app.trading_state.order_quantity.clone();
    let quantity_value = app.trading_state.parse_quantity().unwrap_or(0.0);
    
    // Récupérer les autres champs
    let order_type = app.trading_state.order_type;
    let limit_price = app.trading_state.limit_price.clone();
    let take_profit = app.trading_state.take_profit.clone();
    let stop_loss = app.trading_state.stop_loss.clone();
    let tp_sl_enabled = app.trading_state.tp_sl_enabled;
    
    // Calculer le montant total (quantité * prix)
    let price_for_total = if order_type == OrderType::Limit {
        app.trading_state.parse_limit_price().unwrap_or(current_price)
    } else {
        current_price
    };
    let total_amount = quantity_value * price_for_total;
    
    // Style pour les boutons
    let buy_button_style = move |_theme: &iced::Theme, _status: iced::widget::button::Status| {
        button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.0, 0.7, 0.0))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::from_rgb(0.0, 0.8, 0.0),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    };
    
    let sell_button_style = move |_theme: &iced::Theme, _status: iced::widget::button::Status| {
        button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.7, 0.0, 0.0))),
            text_color: Color::WHITE,
            border: iced::Border {
                color: Color::from_rgb(0.8, 0.0, 0.0),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    };
    
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
    
    container(
        column![
            // En-tête avec symbole et prix
            container(
                column![
                    row![
                        text("Trading")
                            .size(20)
                            .color(colors::TEXT_PRIMARY),
                        Space::new().width(Length::Fill),
                        text(format!("{}", symbol))
                            .size(18)
                            .color(colors::TEXT_PRIMARY)
                            .font(iced::Font::MONOSPACE),
                    ]
                    .width(Length::Fill),
                    Space::new().height(Length::Fixed(8.0)),
                    row![
                        text("Prix actuel:")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fixed(10.0)),
                        text(format_price_for_display(current_price))
                            .size(14)
                            .color(colors::TEXT_PRIMARY)
                            .font(iced::Font::MONOSPACE),
                    ]
                ]
                .spacing(4)
            )
            .style(card_style)
            .padding(12)
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(16.0)),
            
            // Sélection du type d'ordre
            container(
                column![
                    text("Type d'ordre")
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().height(Length::Fixed(8.0)),
                    row![
                        {
                            let is_market = order_type == OrderType::Market;
                            button("MARCHÉ")
                                .on_press(Message::UpdateOrderType(OrderType::Market))
                                .style(move |_theme, _status| {
                                    button::Style {
                                        background: Some(iced::Background::Color(
                                            if is_market {
                                                Color::from_rgb(0.2, 0.4, 0.6)
                                            } else {
                                                Color::from_rgb(0.15, 0.15, 0.2)
                                            }
                                        )),
                                        text_color: Color::WHITE,
                                        border: iced::Border {
                                            color: if is_market {
                                                Color::from_rgb(0.3, 0.5, 0.7)
                                            } else {
                                                Color::from_rgb(0.2, 0.2, 0.25)
                                            },
                                            width: 1.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                })
                                .padding([8, 16])
                                .width(Length::Fill)
                        },
                        Space::new().width(Length::Fixed(8.0)),
                        {
                            let is_limit = order_type == OrderType::Limit;
                            button("LIMITE")
                                .on_press(Message::UpdateOrderType(OrderType::Limit))
                                .style(move |_theme, _status| {
                                    button::Style {
                                        background: Some(iced::Background::Color(
                                            if is_limit {
                                                Color::from_rgb(0.2, 0.4, 0.6)
                                            } else {
                                                Color::from_rgb(0.15, 0.15, 0.2)
                                            }
                                        )),
                                        text_color: Color::WHITE,
                                        border: iced::Border {
                                            color: if is_limit {
                                                Color::from_rgb(0.3, 0.5, 0.7)
                                            } else {
                                                Color::from_rgb(0.2, 0.2, 0.25)
                                            },
                                            width: 1.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                })
                                .padding([8, 16])
                                .width(Length::Fill)
                        },
                    ]
                    .width(Length::Fill),
                ]
                .spacing(4)
            )
            .style(card_style)
            .padding(12)
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(16.0)),
            
            // Formulaire de trading
            container(
                column![
                    text("Quantité")
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().height(Length::Fixed(8.0)),
                    text_input("0.001", &quantity)
                        .on_input(Message::UpdateOrderQuantity)
                        .padding(8)
                        .size(14),
                    Space::new().height(Length::Fixed(8.0)),
                    // Prix limite (visible seulement pour les ordres limit)
                    if order_type == OrderType::Limit {
                        column![
                            text("Prix limite")
                                .size(14)
                                .color(colors::TEXT_PRIMARY),
                            Space::new().height(Length::Fixed(8.0)),
                            text_input(
                                &format_price_for_display(current_price),
                                &limit_price
                            )
                            .on_input(Message::UpdateLimitPrice)
                            .padding(8)
                            .size(14),
                        ]
                        .spacing(4)
                    } else {
                        column![].spacing(0)
                    },
                    Space::new().height(Length::Fixed(8.0)),
                    row![
                        text("Montant total:")
                            .size(12)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text(format!("{} {}", format_price_for_display(total_amount), quote_currency))
                            .size(12)
                            .color(colors::TEXT_PRIMARY)
                            .font(iced::Font::MONOSPACE),
                    ]
                    .width(Length::Fill),
                ]
                .spacing(4)
            )
            .style(card_style)
            .padding(12)
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(16.0)),
            
            // Take Profit et Stop Loss
            container(
                column![
                    text("Take Profit / Stop Loss")
                        .size(14)
                        .color(colors::TEXT_PRIMARY),
                    Space::new().height(Length::Fixed(8.0)),
                    // Afficher la checkbox seulement en mode Market
                    if order_type == OrderType::Market {
                        row![
                            checkbox(tp_sl_enabled)
                                .on_toggle(|_| Message::ToggleTPSLEnabled),
                            text("Activer TP/SL")
                                .size(12)
                                .color(colors::TEXT_SECONDARY),
                        ]
                        .spacing(4)
                        .width(Length::Fill)
                    } else {
                        row![].width(Length::Fill)
                    },
                    Space::new().height(Length::Fixed(8.0)),
                    row![
                        column![
                            text("Take Profit")
                                .size(12)
                                .color(colors::TEXT_SECONDARY),
                            Space::new().height(Length::Fixed(4.0)),
                            text_input("Optionnel", &take_profit)
                                .on_input(Message::UpdateTakeProfit)
                                .padding(8)
                                .size(14),
                        ]
                        .spacing(4)
                        .width(Length::Fill),
                        Space::new().width(Length::Fixed(8.0)),
                        column![
                            text("Stop Loss")
                                .size(12)
                                .color(colors::TEXT_SECONDARY),
                            Space::new().height(Length::Fixed(4.0)),
                            text_input("Optionnel", &stop_loss)
                                .on_input(Message::UpdateStopLoss)
                                .padding(8)
                                .size(14),
                        ]
                        .spacing(4)
                        .width(Length::Fill),
                    ]
                    .width(Length::Fill),
                ]
                .spacing(4)
            )
            .style(card_style)
            .padding(12)
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(16.0)),
            
            // Boutons d'achat et de vente
            row![
                button("ACHETER")
                    .on_press(Message::PlaceBuyOrder)
                    .style(buy_button_style)
                    .padding([12, 24])
                    .width(Length::Fill),
                Space::new().width(Length::Fixed(12.0)),
                button("VENDRE")
                    .on_press(Message::PlaceSellOrder)
                    .style(sell_button_style)
                    .padding([12, 24])
                    .width(Length::Fill),
            ]
            .width(Length::Fill),
            
            Space::new().height(Length::Fixed(16.0)),
            
            // Informations sur le compte
            container(
                column![
                    text("Solde disponible")
                        .size(12)
                        .color(colors::TEXT_SECONDARY),
                    Space::new().height(Length::Fixed(4.0)),
                    text(format!("{:.2} USDT", app.account_info.free_margin))
                        .size(14)
                        .color(colors::TEXT_PRIMARY)
                        .font(iced::Font::MONOSPACE),
                ]
            )
            .style(card_style)
            .padding(12)
            .width(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
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



