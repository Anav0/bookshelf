// src/ui/author_view.rs
use crate::db;
use crate::models::{AuthorModel, BookWithAuthor, NewAuthor};
use crate::ui::{BookshelfApp, Message, Mode, Tab};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length};
use std::collections::HashMap;

// Book statistics struct
#[derive(Debug, Clone, Default)]
struct BookStats {
    bought: usize,
    not_bought: usize,
    finished: usize,
}

// Function to calculate book statistics for all authors
fn calculate_author_stats(books_with_author: &[BookWithAuthor]) -> HashMap<i32, BookStats> {
    let mut stats: HashMap<i32, BookStats> = HashMap::new();

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
pub fn handle_load_authors(app: &mut BookshelfApp) -> Command<Message> {
    Command::perform(
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
) -> Command<Message> {
    match result {
        Ok(authors) => {
            app.authors = authors;
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    Command::none()
}

pub fn handle_add_author_mode(app: &mut BookshelfApp) -> Command<Message> {
    app.mode = Mode::Add;
    app.current_author = None;
    app.author_name = String::new();
    Command::none()
}

pub fn handle_edit_author_mode(app: &mut BookshelfApp, author: AuthorModel) -> Command<Message> {
    app.mode = Mode::Edit;
    app.current_author = Some(author.clone());
    app.author_name = author.Name.unwrap_or_default();
    Command::none()
}

pub fn handle_view_author_mode(app: &mut BookshelfApp) -> Command<Message> {
    app.mode = Mode::View;
    app.current_author = None;
    app.author_books = Vec::new();

    app.update(Message::LoadAuthors)
}

pub fn handle_view_author_details(app: &mut BookshelfApp, author: AuthorModel) -> Command<Message> {
    app.mode = Mode::ViewDetails;
    app.current_author = Some(author.clone());

    // Load books by this author
    Command::perform(
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
) -> Command<Message> {
    match result {
        Ok(books) => {
            app.author_books = books;
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    Command::none()
}

pub fn handle_author_name_changed(app: &mut BookshelfApp, value: String) -> Command<Message> {
    app.author_name = value;
    Command::none()
}

pub fn handle_save_author(app: &mut BookshelfApp) -> Command<Message> {
    let new_author = NewAuthor {
        Name: Some(app.author_name.clone()),
    };

    // Extract author_id outside the closure if we're in edit mode
    let author_id = app.current_author.as_ref().map(|author| author.Id);

    Command::perform(
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
) -> Command<Message> {
    match result {
        Ok(_) => {
            app.mode = Mode::View;
            app.update(Message::LoadAuthors)
        }
        Err(e) => {
            app.error = Some(e);
            Command::none()
        }
    }
}

// New handler for confirming deletion
pub fn handle_confirm_delete_author(
    app: &mut BookshelfApp,
    id: i32,
    name: String,
) -> Command<Message> {
    app.mode = Mode::ConfirmDelete(id, name);
    Command::none()
}

// New handler for canceling deletion
pub fn handle_cancel_delete_author(app: &mut BookshelfApp) -> Command<Message> {
    app.mode = Mode::View;
    Command::none()
}

pub fn handle_delete_author(app: &mut BookshelfApp, id: i32) -> Command<Message> {
    Command::perform(
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
) -> Command<Message> {
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
        .style(iced::theme::Button::Primary);

    let author_list = if app.authors.is_empty() {
        column![text("No authors found").size(16)]
            .spacing(5)
            .width(Length::Fill)
    } else {
        let mut col = column![].spacing(10).width(Length::Fill);

        // Collect book statistics for each author
        let author_stats = calculate_author_stats(&app.books);

        for author in &app.authors {
            let author_name = author
                .Name
                .clone()
                .unwrap_or_else(|| "Unnamed Author".to_string());

            // Get stats for this author
            let stats = author_stats.get(&author.Id).cloned().unwrap_or_default();

            let author_row = row![
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
                    .style(iced::theme::Button::Secondary),
                button("Edit")
                    .on_press(Message::EditAuthorMode(author.clone()))
                    .style(iced::theme::Button::Secondary),
                button("Delete")
                    .on_press(Message::ConfirmDeleteAuthor(
                        author.Id,
                        author.Name.clone().unwrap_or_else(|| "Unnamed Author".to_string())
                    ))
                    .style(iced::theme::Button::Destructive),
            ]
                .spacing(10)
                .align_items(iced::Alignment::Center);

            col = col.push(
                container(author_row)
                    .padding(10)
                    .style(iced::theme::Container::Box),
            );
        }

        col
    };

    column![
        row![
            text("Authors").size(24),
            iced::widget::horizontal_space(Length::Fill),
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

fn view_author_details(app: &BookshelfApp) -> Element<Message> {
    if let Some(author) = &app.current_author {
        let author_name = author
            .Name
            .clone()
            .unwrap_or_else(|| "Unnamed Author".to_string());

        let back_button = button("Back to Authors")
            .on_press(Message::ViewAuthorMode)
            .style(iced::theme::Button::Secondary);

        let edit_button = button("Edit Author")
            .on_press(Message::EditAuthorMode(author.clone()))
            .style(iced::theme::Button::Primary);

        let delete_button = button("Delete Author")
            .on_press(Message::ConfirmDeleteAuthor(
                author.Id,
                author.Name.clone().unwrap_or_else(|| "Unnamed Author".to_string())
            ))
            .style(iced::theme::Button::Destructive);

        let header = row![
            text(format!("Author: {}", author_name)).size(24),
            iced::widget::horizontal_space(Length::Fill),
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
            let mut col = column![
                text(format!("Books by {} ({})", author_name, book_count)).size(20)
            ]
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
                        row![
                            text(price_text).size(14),
                            text(status_text).size(14)
                        ]
                        .spacing(10)
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                    button("View in Books")
                        .on_press(Message::TabSelected(crate::ui::Tab::Books))
                        .style(iced::theme::Button::Secondary)
                        .padding(8),
                ]
                    .spacing(15)
                    .padding(10)
                    .align_items(iced::Alignment::Center);

                col = col.push(
                    container(book_row)
                        .padding(10)
                        .style(iced::theme::Container::Box),
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
                .style(iced::theme::Button::Primary),
            button("Cancel")
                .on_press(Message::ViewAuthorMode)
                .style(iced::theme::Button::Secondary),
        ]
        .spacing(10)
    ]
        .spacing(10)
        .padding(20)
        .max_width(500);

    container(form)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .into()
}

// New function to display deletion confirmation
fn view_delete_confirmation<'a>(app: &'a BookshelfApp, id: i32, name: &str) -> Element<'a, Message> {
    let confirmation = column![
        text(format!("Are you sure you want to delete the author:")).size(20),
        text(format!("\"{}\"?", name))
            .size(24)
            .style(iced::Color::from_rgb(0.8, 0.0, 0.0)),
        text("This action cannot be undone.").size(16),
        if !app.author_books.is_empty() {
            text(format!("Warning: This author has {} books associated with them.", app.author_books.len()))
                .size(16)
                .style(iced::Color::from_rgb(0.8, 0.6, 0.0))
        } else {
            text("")
        },
        row![
            button("Cancel")
                .on_press(Message::CancelDeleteAuthor)
                .style(iced::theme::Button::Secondary)
                .padding(10)
                .width(Length::Fill),
            button("Confirm Delete")
                .on_press(Message::DeleteAuthor(id))
                .style(iced::theme::Button::Destructive)
                .padding(10)
                .width(Length::Fill),
        ]
        .spacing(20)
        .padding(20)
    ]
        .spacing(20)
        .padding(30)
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);

    container(confirmation)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Box)
        .into()
}