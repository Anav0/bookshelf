// src/ui/common.rs
use crate::ui::author_view;
use crate::ui::book_view;
use crate::ui::{BookshelfApp, Message, Tab};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Color, Element, Length};

pub fn view(app: &BookshelfApp) -> Element<Message> {
    // Tabs navigation
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

    // Error messages
    let error_message = if let Some(error) = &app.error {
        container(text(error).style(Color::from_rgb(0.8, 0.0, 0.0)).size(14))
            .padding(10)
            .width(Length::Fill)
    } else {
        container(text("")).width(Length::Fill)
    };

    // Only show search bar in Books tab
    let search_bar = if matches!(app.current_tab, Tab::Books) {
        let search_placeholder = "Search by title, author, or price...";

        container(
            row![
                text_input(search_placeholder, &app.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .on_submit(Message::PerformSearch)
                    .padding(10)
                    .width(Length::Fill),
                button("Search")
                    .on_press(Message::PerformSearch)
                    .style(iced::theme::Button::Primary)
                    .padding(8),
                if !app.search_query.is_empty() {
                    button("Clear")
                        .on_press(Message::ClearSearch)
                        .style(iced::theme::Button::Secondary)
                        .padding(8)
                } else {
                    button("Clear")
                        .style(iced::theme::Button::Secondary)
                        .padding(8)
                }
            ]
            .spacing(10)
            .padding(10)
            .width(Length::Fill),
        )
    } else {
        // Empty container for Authors tab - now of the same type as the 'if' branch
        container(row![].width(Length::Fill))
    };

    // Main content
    let content = match app.current_tab {
        Tab::Books => book_view::view(app),
        Tab::Authors => author_view::view(app),
    };

    column![tab_row, error_message, search_bar, content,].into()
}
