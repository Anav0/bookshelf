// src/ui/common.rs
use crate::ui::book_view;
use crate::ui::{author_view, LIST_PADDING, LIST_SPACING};
use crate::ui::{BookshelfApp, Message, SortDirection, SortField, Tab};
use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &BookshelfApp) -> Element<Message> {
    // Tabs navigation
    let tab_row = row![
        button(text("Books").size(20))
            .on_press(Message::TabSelected(Tab::Books))
            .style(if matches!(app.current_tab, Tab::Books) {
                button::primary
            } else {
                button::secondary
            }),
        button(text("Authors").size(20))
            .on_press(Message::TabSelected(Tab::Authors))
            .style(if matches!(app.current_tab, Tab::Authors) {
                button::primary
            } else {
                button::secondary
            }),
    ]
    .spacing(LIST_SPACING)
    .padding(LIST_PADDING);

    // Error messages
    let error_message = if let Some(error) = &app.error {
        container(text(error).size(14))
            .padding(10)
            .width(Length::Fill)
    } else {
        container(text("")).width(Length::Fill)
    };

    // Only show search and sort options in Books tab
    let top_bar = if matches!(app.current_tab, Tab::Books) {
        let search_placeholder = "Search by title, author, or price...";

        column![
            // Search bar
            container(
                row![
                    text_input(search_placeholder, &app.search_query)
                        .on_input(Message::SearchQueryChanged)
                        .on_submit(Message::PerformSearch)
                        .padding(10)
                        .width(Length::Fill),
                    button("Search")
                        .on_press(Message::PerformSearch)
                        .style(button::primary)
                        .padding(8),
                    if !app.search_query.is_empty() {
                        button("Clear")
                            .on_press(Message::ClearSearch)
                            .style(button::secondary)
                            .padding(8)
                    } else {
                        button("Clear")
                            .style(button::secondary)
                            .padding(8)
                    }
                ]
                .spacing(LIST_SPACING)
                .padding(LIST_PADDING)
                .width(Length::Fill)
            ),
            // Sort options
            container(
                row![
                    text("Sort by:").size(14),
                    pick_list(
                        vec![
                            SortField::Title,
                            SortField::Author,
                            SortField::Price,
                            SortField::DateAdded
                        ],
                        Some(app.sort_field.clone()),
                        Message::SortFieldSelected
                    )
                    .padding(8)
                    .width(Length::FillPortion(3)),
                    pick_list(
                        vec![SortDirection::Ascending, SortDirection::Descending],
                        Some(app.sort_direction.clone()),
                        Message::SortDirectionSelected
                    )
                    .padding(8)
                    .width(Length::FillPortion(3)) // Remove the Apply button
                ]
                .spacing(LIST_SPACING)
                .padding(LIST_PADDING)
                .width(Length::Fill)
            )
        ]
    } else {
        // Empty container for Authors tab
        column![container(row![]).width(Length::Fill).height(Length::Shrink)]
    };

    // Main content
    let content = match app.current_tab {
        Tab::Books => book_view::view(app),
        Tab::Authors => author_view::view(app),
    };

    column![tab_row, error_message, top_bar, content,].into()
}
