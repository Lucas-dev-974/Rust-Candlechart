//! Point d'entrée principal de l'application
//!
//! Ce fichier initialise et lance l'application Iced.

mod finance_chart;
mod app;

use app::ChartApp;

fn main() -> iced::Result {
    iced::daemon(ChartApp::new, ChartApp::update, ChartApp::view)
        .title(ChartApp::title)
        .theme(ChartApp::theme)
        .subscription(ChartApp::subscription)
        .run()
}
