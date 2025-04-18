// src/main.rs
mod models;
mod schema;
mod db;
mod ui;

use iced::{executor, Application, Command, Element, Settings, Theme};
use iced::window::{Level, PlatformSpecific};
use crate::ui::BookshelfApp;

fn main() -> iced::Result {
    dotenv::dotenv().ok();

    BookshelfApp::run(Settings {
        window: iced::window::Settings {
            size: (1024, 768),
            position: iced::window::Position::Centered,
            min_size: Some((800, 600)),
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            icon: None,
            level: Level::Normal,
            platform_specific: PlatformSpecific::default(),
        },
        default_text_size: 16.0,
        antialiasing: true,
        exit_on_close_request: true,
        ..Settings::default()
    })
}
