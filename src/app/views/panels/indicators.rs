//! Panneaux d'indicateurs (Volume, RSI, MACD)
//!
//! Ce module gère l'affichage des panneaux d'indicateurs sous le graphique principal.

use iced::{Element, Length};
use iced::widget::{row, column, mouse_area};
use crate::app::{app_state::ChartApp, messages::Message};
use crate::finance_chart::{volume_chart, rsi_chart, macd_chart, volume_y_axis, rsi_y_axis, macd_y_axis, scale::VolumeScale};
use crate::app::ui::{volume_resize_handle, rsi_panel_resize_handle, macd_panel_resize_handle};

/// Construit et ajoute les panneaux d'indicateurs visibles au layout
///
/// Cette fonction vérifie quels panneaux d'indicateurs sont visibles
/// (Volume, RSI, MACD) et les ajoute à la liste des éléments de layout.
pub fn build_indicator_panels<'a>(app: &'a ChartApp, layout_items: &mut Vec<Element<'a, Message>>) {
    // Panneau Volume
    if app.ui.panels.volume.visible {
        let handle_height = 6.0;
        let chart_height = app.ui.panels.volume.size - handle_height;
        
        // Créer une échelle de volume basée sur les données visibles
        let volume_scale = {
            // Obtenir la plage temporelle visible depuis le viewport
            let (min_time, max_time) = app.chart_state.viewport.time_scale().time_range();
            let time_range = min_time..max_time;
            
            // Utiliser volume_range_for_time_range pour obtenir le min/max des bougies visibles
            // mais forcer le min à 0 pour une meilleure visualisation (barres depuis le bas)
            if let Some((_min_volume, max_volume)) = app.chart_state.series_manager
                .active_series()
                .next()
                .and_then(|series| series.data.volume_range_for_time_range(time_range)) {
                // Forcer le minimum à 0 pour que les barres partent du bas
                VolumeScale::new(0.0, max_volume, chart_height)
            } else {
                // Valeurs par défaut si aucune donnée
                VolumeScale::new(0.0, 1000.0, chart_height)
            }
        };

        let volume_panel = volume_chart(&app.chart_state, volume_scale.clone());
        let volume_y_axis_panel = volume_y_axis(volume_scale);
        
        // Créer une row avec le graphique de volume + son axe Y
        let volume_chart_row = row![
            volume_panel,
            volume_y_axis_panel
        ]
        .width(Length::Fill)
        .height(Length::Fixed(chart_height));
        
        // Ajouter le handle de redimensionnement en haut
        let volume_panel_with_handle = mouse_area(
            column![
                volume_resize_handle(handle_height, app.ui.panels.volume.is_resizing),
                volume_chart_row
            ]
            .width(Length::Fill)
            .height(Length::Fixed(app.ui.panels.volume.size))
        )
        .on_enter(Message::SetVolumePanelFocus(true))
        .on_exit(Message::SetVolumePanelFocus(false));
        
        layout_items.push(volume_panel_with_handle.into());
    }

    // Panneau RSI
    if app.ui.panels.rsi.visible {
        let handle_height = 6.0;
        let chart_height = app.ui.panels.rsi.size - handle_height;
        
        let rsi_panel = rsi_chart(&app.chart_state);
        let rsi_y_axis_panel = rsi_y_axis(&app.chart_state, chart_height);
        
        // Créer une row avec le graphique RSI + son axe Y
        let rsi_chart_row = row![
            rsi_panel,
            rsi_y_axis_panel
        ]
        .width(Length::Fill)
        .height(Length::Fixed(chart_height));
        
        // Ajouter le handle de redimensionnement en haut
        let rsi_panel_with_handle = mouse_area(
            column![
                rsi_panel_resize_handle(handle_height, app.ui.panels.rsi.is_resizing),
                rsi_chart_row
            ]
            .width(Length::Fill)
            .height(Length::Fixed(app.ui.panels.rsi.size))
        )
        .on_enter(Message::SetRSIPanelFocus(true))
        .on_exit(Message::SetRSIPanelFocus(false));
        
        layout_items.push(rsi_panel_with_handle.into());
    }

    // Panneau MACD
    if app.ui.panels.macd.visible {
        let handle_height = 6.0;
        let chart_height = app.ui.panels.macd.size - handle_height;
        
        let macd_panel = macd_chart(&app.chart_state);
        let macd_y_axis_panel = macd_y_axis(&app.chart_state);
        
        // Créer une row avec le graphique MACD + son axe Y
        let macd_chart_row = row![
            macd_panel,
            macd_y_axis_panel
        ]
        .width(Length::Fill)
        .height(Length::Fixed(chart_height));
        
        // Ajouter le handle de redimensionnement en haut
        let macd_panel_with_handle = mouse_area(
            column![
                macd_panel_resize_handle(handle_height, app.ui.panels.macd.is_resizing),
                macd_chart_row
            ]
            .width(Length::Fill)
            .height(Length::Fixed(app.ui.panels.macd.size))
        )
        .on_enter(Message::SetMACDPanelFocus(true))
        .on_exit(Message::SetMACDPanelFocus(false));
        
        layout_items.push(macd_panel_with_handle.into());
    }
}