// src/ui/author_view.rs
use crate::db;
use crate::models::{AuthorModel, BookWithAuthor, NewAuthor, ID};
use crate::ui::components::searchable_dropdown::SearchableDropdown;
use crate::ui::{BookshelfApp, Message, Mode};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Row};
use iced::Fill;
use iced::{Element, Length};
use std::collections::HashMap;

// Book statistics struct
#[derive(Debug, Clone, Default)]
struct BookStats {
    bought: usize,
    not_bought: usize,
    finished: usize,
}

// Function to calculate book statistics for all authors
fn calculate_author_stats(books_with_author: &[BookWithAuthor]) -> HashMap<ID, BookStats> {
    let mut stats: HashMap<ID, BookStats> = HashMap::new();

    for pair in books_with_author {
        if let Some(author_id) = pair.book.AuthorFK {
            let stat = stats.entry(author_id).or_default();
            if pair.book.bought.is_some() {
                stat.bought += 1;
            } else {
                stat.not_bought += 1;
            }

            if pair.book.finished.is_some() {
                stat.finished += 1;
            }
        }
    }

    stats
}

// Handler functions for author-related messages
pub fn handle_load_authors(_: &mut BookshelfApp) -> iced::Task<Message> {
    iced::Task::perform(
        async {
            match db::get_authors() {
                Ok(authors) => Ok(authors),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::AuthorsLoaded,
    )
}

pub fn handle_authors_loaded(
    app: &mut BookshelfApp,
    result: Result<Vec<AuthorModel>, String>,
) -> iced::Task<Message> {
    match result {
        Ok(authors) => {
            app.authors = authors.clone();
            app.author_dropdown = SearchableDropdown::new(authors, app.selected_author.clone());
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    iced::Task::none()
}

pub fn handle_add_author_mode(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::Add;
    app.current_author = None;
    app.author_name = String::new();
    iced::Task::none()
}

pub fn handle_edit_author_mode(app: &mut BookshelfApp, author: AuthorModel) -> iced::Task<Message> {
    app.mode = Mode::Edit;
    app.current_author = Some(author.clone());
    app.author_name = author.Name.unwrap_or_default();
    iced::Task::none()
}

pub fn handle_view_author_mode(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::View;
    app.current_author = None;
    app.author_books = Vec::new();

    app.update(Message::LoadAuthors)
}

pub fn handle_view_author_details(
    app: &mut BookshelfApp,
    author: AuthorModel,
) -> iced::Task<Message> {
    app.mode = Mode::ViewDetails;
    app.current_author = Some(author.clone());

    // Load books by this author
    iced::Task::perform(
        async move {
            match db::get_books_by_author(author.Id) {
                Ok(books) => Ok(books),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::AuthorBooksLoaded,
    )
}

pub fn handle_author_books_loaded(
    app: &mut BookshelfApp,
    result: Result<Vec<BookWithAuthor>, String>,
) -> iced::Task<Message> {
    match result {
        Ok(books) => {
            app.author_books = books;
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    iced::Task::none()
}

pub fn handle_author_name_changed(app: &mut BookshelfApp, value: String) -> iced::Task<Message> {
    app.author_name = value;
    iced::Task::none()
}

pub fn handle_save_author(app: &mut BookshelfApp) -> iced::Task<Message> {
    let new_author = NewAuthor {
        Name: Some(app.author_name.clone()),
    };

    // Extract author_id outside the closure if we're in edit mode
    let author_id = app.current_author.as_ref().map(|author| author.Id);

    iced::Task::perform(
        async move {
            // Use author_id that we extracted before the closure
            if let Some(id) = author_id {
                match db::update_author(id, &new_author) {
                    Ok(updated) => Ok(updated),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                match db::create_author(&new_author) {
                    Ok(created) => Ok(created),
                    Err(e) => Err(e.to_string()),
                }
            }
        },
        Message::AuthorSaved,
    )
}

pub fn handle_author_saved(
    app: &mut BookshelfApp,
    result: Result<AuthorModel, String>,
) -> iced::Task<Message> {
    match result {
        Ok(_) => {
            app.mode = Mode::View;
            app.update(Message::LoadAuthors)
        }
        Err(e) => {
            app.error = Some(e);
            iced::Task::none()
        }
    }
}

// New handler for confirming deletion
pub fn handle_confirm_delete_author(
    app: &mut BookshelfApp,
    id: ID,
    name: String,
) -> iced::Task<Message> {
    app.mode = Mode::ConfirmDelete(id, name);
    iced::Task::none()
}

// New handler for canceling deletion
pub fn handle_cancel_delete_author(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::View;
    iced::Task::none()
}

pub fn handle_delete_author(_: &mut BookshelfApp, id: ID) ->
                                                         iced::Task<Message> {
    iced::Task::perform(
        async move {
            match db::delete_author(id) {
                Ok(count) => Ok(count),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::AuthorDeleted,
    )
}

pub fn handle_author_deleted(
    app: &mut BookshelfApp,
    result: Result<usize, String>,
) -> iced::Task<Message> {
    app.mode = Mode::View; // Ensure we go back to view mode

    match result {
        Ok(_) => app.update(Message::LoadAuthors),
        Err(e) => {
            app.error = Some(e);
            app.update(Message::LoadAuthors) // Always go back to author list even on error
        }
    }
}

// View functions for authors
pub fn view(app: &BookshelfApp) -> Element<Message> {
    match app.mode {
        Mode::View => view_author_list(app),
        Mode::ViewDetails => view_author_details(app),
        Mode::Add | Mode::Edit => view_author_form(app),
        Mode::ConfirmDelete(id, ref name) => view_delete_confirmation(app, id, name),
    }
}

fn view_author_list(app: &BookshelfApp) -> Element<Message> {
    let add_button = button("Add New Author")
        .on_press(Message::AddAuthorMode)
        .style(button::primary);

    let author_list = if app.authors.is_empty() {
        column![text("No authors found").size(16)]
            .spacing(5)
            .width(Length::Fill)
    } else {
        create_authors_list(app)
    };

    column![
        row![
            text("Authors").size(24),
            iced::widget::horizontal_space(),
            add_button
        ]
        .padding(10)
        .width(Length::Fill),
        scrollable(container(author_list).padding(10).width(Length::Fill)).height(Length::Fill)
    ]
    .spacing(20)
    .padding(20)
    .into()
}

fn create_authors_list<'a>(app: &BookshelfApp) -> Column<Message> {
    let mut list = column![].spacing(10).width(Length::Fill);

    let author_stats = calculate_author_stats(&app.books);

    for author in &app.authors {
        list = list.push(
            container(create_author_row(&author_stats, author))
                .padding(10)
                .style(container::bordered_box),
        );
    }

    list
}

fn create_author_row<'a>(
    author_stats: &HashMap<ID, BookStats>,
    author: &AuthorModel,
) -> Row<'a, Message> {
    let author_name = author
        .Name
        .clone()
        .unwrap_or_else(|| "Unnamed Author".to_string());

    let stats = author_stats.get(&author.Id).cloned().unwrap_or_default();

    row![
        column![
            text(author_name).size(18),
            row![
                text(format!("Bought: {}", stats.bought)).size(14),
                text(format!("Not bought: {}", stats.not_bought)).size(14),
                text(format!("Finished: {}", stats.finished)).size(14),
            ]
            .spacing(10)
        ]
        .spacing(5)
        .width(Length::Fill),
        button("View")
            .on_press(Message::ViewAuthorDetails(author.clone()))
            .style(button::secondary),
        button("Edit")
            .on_press(Message::EditAuthorMode(author.clone()))
            .style(button::secondary),
        button("Delete")
            .on_press(Message::ConfirmDeleteAuthor(
                author.Id,
                author
                    .Name
                    .clone()
                    .unwrap_or_else(|| "Unnamed Author".to_string())
            ))
            .style(button::danger),
    ]
    .spacing(10)
    .align_y(iced::alignment::Vertical::Center)
}

fn view_author_details(app: &BookshelfApp) -> Element<Message> {
    if let Some(author) = &app.current_author {
        let author_name = author
            .Name
            .clone()
            .unwrap_or_else(|| "Unnamed Author".to_string());

        let back_button = button("Back to Authors")
            .on_press(Message::ViewAuthorMode)
            .style(button::secondary);

        let edit_button = button("Edit Author")
            .on_press(Message::EditAuthorMode(author.clone()))
            .style(button::primary);

        let delete_button = button("Delete Author")
            .on_press(Message::ConfirmDeleteAuthor(
                author.Id,
                author
                    .Name
                    .clone()
                    .unwrap_or_else(|| "Unnamed Author".to_string()),
            ))
            .style(button::danger);

        let header = row![
            text(format!("Author: {}", author_name)).size(24),
            iced::widget::horizontal_space(),
            back_button,
            edit_button,
            delete_button,
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fill);

        let book_count = app.author_books.len();
        let book_list = if book_count == 0 {
            column![text("No books found for this author").size(16)]
                .spacing(5)
                .width(Length::Fill)
                .padding(20)
        } else {
            let mut col =
                column![text(format!("Books by {} ({})", author_name, book_count)).size(20)]
                    .spacing(15)
                    .width(Length::Fill)
                    .padding(20);

            for pair in &app.author_books {
                let price_text = pair
                    .book
                    .price
                    .map(|p| format!("{:.2}zł", p))
                    .unwrap_or_else(|| "No price".to_string());

                let status_text = {
                    let mut statuses = Vec::new();

                    if pair.book.bought.is_some() {
                        statuses.push("Bought");
                    } else {
                        statuses.push("Not bought");
                    }

                    if pair.book.finished.is_some() {
                        statuses.push("Finished");
                    }

                    statuses.join(" · ")
                };

                let book_row = row![
                    column![
                        text(&pair.book.title).size(18),
                        row![text(price_text).size(14), text(status_text).size(14)].spacing(10)
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                    button("View in Books")
                        .on_press(Message::TabSelected(crate::ui::Tab::Books))
                        .style(button::secondary)
                        .padding(8),
                ]
                .spacing(15)
                .padding(10)
                .align_y(iced::alignment::Vertical::Center);

                col = col.push(
                    container(book_row)
                        .padding(10)
                        .style(container::bordered_box),
                );
            }

            col
        };

        column![
            header,
            scrollable(container(book_list).width(Length::Fill)).height(Length::Fill)
        ]
        .spacing(20)
        .padding(20)
        .into()
    } else {
        // Fallback in case no author is selected
        view_author_list(app)
    }
}

fn view_author_form(app: &BookshelfApp) -> Element<Message> {
    let title = match app.mode {
        Mode::Add => "Add New Author",
        Mode::Edit => "Edit Author",
        _ => unreachable!(),
    };

    let form = column![
        text(title).size(24),
        text("Name:").size(16),
        text_input("Enter author name", &app.author_name)
            .on_input(Message::AuthorNameChanged)
            .padding(10),
        row![
            button("Save")
                .on_press(Message::SaveAuthor)
                .style(button::primary),
            button("Cancel")
                .on_press(Message::ViewAuthorMode)
                .style(button::secondary),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .padding(20)
    .max_width(500);

    container(form)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

// New function to display deletion confirmation
fn view_delete_confirmation<'a>(
    app: &'a BookshelfApp,
    id: ID,
    name: &str,
) -> Element<'a, Message> {
    let confirmation = column![
        text(format!("Are you sure you want to delete the author:")).size(20),
        text(format!("\"{}\"?", name)).size(24),
        text("This action cannot be undone.").size(16),
        if !app.author_books.is_empty() {
            text(format!(
                "Warning: This author has {} books associated with them.",
                app.author_books.len()
            ))
            .size(16)
        } else {
            text("")
        },
        row![
            button("Cancel")
                .on_press(Message::CancelDeleteAuthor)
                .style(button::secondary)
                .padding(10)
                .width(Length::Fill),
            button("Confirm Delete")
                .on_press(Message::DeleteAuthor(id))
                .style(button::danger)
                .padding(10)
                .width(Length::Fill),
        ]
        .spacing(20)
        .padding(20)
    ]
    .spacing(20)
    .padding(30)
    .width(Length::Fill)
    .align_x(iced::Alignment::Center);

    container(confirmation)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Fill)
        .center_y(Fill)
        .style(container::bordered_box)
        .into()
}
