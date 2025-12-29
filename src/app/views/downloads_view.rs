//! Vue de la fenêtre de téléchargements

use iced::widget::{column, container, row, text, progress_bar, Space, scrollable, button};
use iced::{Element, Length, Color};
use crate::app::app_state::ChartApp;
use crate::app::messages::Message;
use crate::app::view_styles;

/// Vue de la fenêtre de téléchargements
pub fn view_downloads(app: &ChartApp) -> Element<'_, Message> {
    let downloads = app.download_manager.all_downloads();
    
    if downloads.is_empty() {
        container(
            column![
                text("Aucun téléchargement en cours")
                    .size(16)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
            ]
            .spacing(20)
            .padding(40)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        let mut download_list = column![]
            .spacing(15)
            .padding(20);
        
        for (series_id, progress) in downloads.iter() {
            let series_name = app.chart_state.series_manager
                .get_series(series_id)
                .map(|s| s.full_name())
                .unwrap_or_else(|| series_id.name.clone());
            
            let progress_value = if progress.estimated_total > 0 {
                Some(progress.current_count as f32 / progress.estimated_total as f32)
            } else {
                None
            };
            
            let progress_text = if progress.estimated_total > 0 {
                format!("{} / {} bougies", progress.current_count, progress.estimated_total)
            } else {
                format!("{} bougies", progress.current_count)
            };
            
            let gaps_text = if progress.gaps_remaining.is_empty() {
                "Dernier gap".to_string()
            } else {
                format!("{} gap(s) restant(s)", progress.gaps_remaining.len() + 1)
            };
            
            // Cloner les valeurs pour éviter les problèmes de lifetime
            let series_name_clone = series_name.clone();
            let gaps_text_clone = gaps_text.clone();
            let progress_text_clone = progress_text.clone();
            
            let is_paused = progress.paused;
            let series_id_clone = series_id.clone();
            
            download_list = download_list.push(
                container(
                    column![
                        row![
                            text(series_name_clone.clone())
                                .size(14)
                                .color(Color::from_rgb(0.9, 0.9, 0.9)),
                            Space::new().width(Length::Fill),
                            if is_paused {
                                text("⏸️ En pause")
                                    .size(12)
                                    .color(Color::from_rgb(1.0, 0.7, 0.0))
                            } else {
                                text(gaps_text_clone.clone())
                                    .size(12)
                                    .color(Color::from_rgb(0.7, 0.7, 0.7))
                            },
                        ]
                        .width(Length::Fill),
                        Space::new().width(Length::Fixed(8.0)),
                        progress_bar(0.0..=1.0, progress_value.unwrap_or(0.0)),
                        Space::new().width(Length::Fixed(4.0)),
                        row![
                            text(progress_text_clone.clone())
                                .size(11)
                                .color(Color::from_rgb(0.6, 0.6, 0.6)),
                            Space::new().width(Length::Fill),
                            if is_paused {
                                button("▶️ Reprendre")
                                    .on_press(Message::ResumeDownload(series_id_clone.clone()))
                                    .style(view_styles::icon_button_style)
                            } else {
                                button("⏸️ Pause")
                                    .on_press(Message::PauseDownload(series_id_clone.clone()))
                                    .style(view_styles::icon_button_style)
                            },
                            Space::new().width(Length::Fixed(5.0)),
                            button("⏹️ Arrêter")
                                .on_press(Message::StopDownload(series_id_clone.clone()))
                                .style(view_styles::icon_button_style),
                        ]
                        .width(Length::Fill)
                        .align_y(iced::Alignment::Center),
                    ]
                    .spacing(5)
                )
                .padding(15)
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))),
                        border: iced::Border {
                            color: Color::from_rgba(0.4, 0.4, 0.4, 0.3),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    }
                })
            );
        }
        
        container(
            scrollable(
                column![
                    text("Téléchargements en cours")
                        .size(18)
                        .color(Color::from_rgb(0.9, 0.9, 0.9)),
                    Space::new().width(Length::Fixed(20.0)),
                    download_list,
                ]
                .padding(20)
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

