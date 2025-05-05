mod db;
mod models;
mod schema;
mod ui;

use crate::ui::{BookshelfApp, Message};
use iced::window::icon::from_file_data;
use iced::{window, Size};

fn main() -> iced::Result {
    dotenv::dotenv().ok();

    let icon = from_file_data(include_bytes!("assets/icon.png"), None).ok();

    // Create window settings
    let window_settings = window::Settings {
        size: Size::new(1024.0, 768.0), // Note: now uses Size struct
        position: window::Position::Centered,
        min_size: Some(Size::new(800.0, 600.0)), // Note: uses Size struct
        max_size: None,
        visible: true,
        resizable: true,
        decorations: true,
        transparent: false,
        level: Default::default(),
        icon,
        platform_specific: Default::default(),
        exit_on_close_request: false,
    };

    // New application initialization approach
    iced::application("Bookshelf App", BookshelfApp::update, BookshelfApp::view)
        .window(window_settings)
        .antialiasing(true)
        .exit_on_close_request(true)
        .run_with(|| {
            (
                BookshelfApp::new(), // Initialize your app state
                iced::Task::perform(async {}, |_| Message::Initialize),
            )
        })
}
