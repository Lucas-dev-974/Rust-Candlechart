//! Section "Backtest"

use iced::widget::{button, column, container, row, text, Space};
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
    
    let mut content = column![]
        .spacing(15)
        .padding(20);
    
    // Titre
    content = content.push(
        text("Backtest")
            .size(18)
            .color(colors::TEXT_PRIMARY)
    );
    
    // Informations sur la date sélectionnée
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
    
    // Boutons de contrôle
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
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}


