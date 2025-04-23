// src/ui/common.rs
use crate::ui::author_view;
use crate::ui::book_view;
use crate::ui::{BookshelfApp, Message, Tab};
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

pub fn view(app: &BookshelfApp) -> Element<Message> {
    let tab_row = row![
        button(text("Books").size(20))
            .on_press(Message::TabSelected(Tab::Books))
            .style(if matches!(app.current_tab, Tab::Books) {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            }),
        button(text("Authors").size(20))
            .on_press(Message::TabSelected(Tab::Authors))
            .style(if matches!(app.current_tab, Tab::Authors) {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            }),
    ]
    .spacing(10)
    .padding(10);

    let content = match app.current_tab {
        Tab::Books => book_view::view(app),
        Tab::Authors => author_view::view(app),
    };

    let error_message = if let Some(error) = &app.error {
        container(text(error).style(Color::from_rgb(0.8, 0.0, 0.0)).size(14))
            .padding(10)
            .width(Length::Fill)
    } else {
        container(text("")).width(Length::Fill)
    };

    column![tab_row, error_message, content,].into()
}
