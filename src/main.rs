mod db;
mod models;
mod schema;
mod ui;

use crate::ui::BookshelfApp;
use iced::futures::future::ok;
use iced::window::icon::from_file_data;
use iced::{window, Application, Settings};

fn main() -> iced::Result {
    dotenv::dotenv().ok();

    let icon = from_file_data(include_bytes!("assets/icon.png"), None).ok();

    BookshelfApp::run(Settings {
        window: window::Settings {
            size: (1024, 768),
            position: window::Position::Centered,
            min_size: Some((800, 600)),
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            icon,
            level: window::Level::Normal,
            platform_specific: window::PlatformSpecific::default(),
        },
        default_text_size: 16.0,
        antialiasing: true,
        exit_on_close_request: true,
        ..Settings::default()
    })
}
