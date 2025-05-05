// src/ui/book_view.rs
use crate::db;
use crate::models::{BookModel, BookWithAuthor, NewBook};
use crate::ui::components::searchable_dropdown;
use crate::ui::{sort_books, BookshelfApp, Message, Mode, LIST_MAX_WIDTH};
use chrono::{Local, NaiveDateTime};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Element, Length};

// Handler functions for book-related messages
pub fn handle_load_books(_: &mut BookshelfApp) -> iced::Task<Message> {
    iced::Task::perform(
        async {
            match db::get_books() {
                Ok(books) => Ok(books),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::BooksLoaded,
    )
}

pub fn handle_add_book_mode(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::Add;
    app.current_book = None;
    app.book_title = String::new();
    app.book_price = String::new();
    app.book_bought_date = String::new();
    app.book_finished_date = String::new();
    app.selected_author = None;

    iced::Task::perform(async {}, |_| Message::LoadAuthors)
}

pub fn handle_edit_book_mode(app: &mut BookshelfApp, book: BookWithAuthor) -> iced::Task<Message> {
    app.mode = Mode::Edit;
    app.current_book = Some(book.clone());
    app.book_title = book.book.title;
    app.book_price = book.book.price.map_or_else(String::new, |p| p.to_string());
    app.book_bought_date = book
        .book
        .bought
        .map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    app.book_finished_date = book
        .book
        .finished
        .map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    app.selected_author = book.author;

    iced::Task::perform(async {}, |_| Message::LoadAuthors)
}

pub fn handle_view_book_mode(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::View;
    app.current_book = None;

    app.update(Message::LoadBooks)
}

pub fn handle_book_title_changed(app: &mut BookshelfApp, value: String) -> iced::Task<Message> {
    app.book_title = value;
    iced::Task::none()
}

pub fn handle_book_price_changed(app: &mut BookshelfApp, value: String) -> iced::Task<Message> {
    app.book_price = value;
    iced::Task::none()
}

pub fn handle_book_bought_date_changed(
    app: &mut BookshelfApp,
    value: String,
) -> iced::Task<Message> {
    app.book_bought_date = value;
    iced::Task::none()
}

pub fn handle_book_finished_date_changed(
    app: &mut BookshelfApp,
    value: String,
) -> iced::Task<Message> {
    app.book_finished_date = value;
    iced::Task::none()
}

pub fn handle_save_book(app: &mut BookshelfApp) -> iced::Task<Message> {
    let price = if app.book_price.is_empty() {
        None
    } else {
        match app.book_price.parse::<f32>() {
            Ok(p) => Some(p),
            Err(_) => {
                app.error = Some("Invalid price format".to_string());
                return iced::Task::none();
            }
        }
    };

    let parse_datetime = |s: &str| -> Option<NaiveDateTime> {
        if s.is_empty() {
            None
        } else {
            match NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                Ok(dt) => Some(dt),
                Err(_) => None, // Handle date parsing error
            }
        }
    };

    let bought_date = parse_datetime(&app.book_bought_date);
    let finished_date = parse_datetime(&app.book_finished_date);

    let now = Local::now().naive_local();
    let added_date = app
        .current_book
        .as_ref()
        .and_then(|b| b.book.added)
        .unwrap_or(now);

    // Extract book_id outside the closure if we're in edit mode
    let book_id = app.current_book.as_ref().map(|book| book.book.id);

    let new_book = NewBook {
        title: app.book_title.clone(),
        price,
        bought: bought_date,
        finished: finished_date,
        added: Some(added_date),
        AuthorFK: app.selected_author.as_ref().map(|a| a.Id),
    };

    iced::Task::perform(
        async move {
            // Use book_id that we extracted before the closure
            if let Some(id) = book_id {
                match db::update_book(id, &new_book) {
                    Ok(updated) => Ok(updated),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                match db::create_book(&new_book) {
                    Ok(created) => Ok(created),
                    Err(e) => Err(e.to_string()),
                }
            }
        },
        Message::BookSaved,
    )
}

pub fn handle_book_saved(
    app: &mut BookshelfApp,
    result: Result<BookModel, String>,
) -> iced::Task<Message> {
    match result {
        Ok(_) => {
            app.mode = Mode::View;
            app.update(Message::LoadBooks)
        }
        Err(e) => {
            app.error = Some(e);
            iced::Task::none()
        }
    }
}

// New handler for confirming deletion
pub fn handle_confirm_delete_book(
    app: &mut BookshelfApp,
    id: i32,
    title: String,
) -> iced::Task<Message> {
    app.mode = Mode::ConfirmDelete(id, title);
    iced::Task::none()
}

// New handler for canceling deletion
pub fn handle_cancel_delete_book(app: &mut BookshelfApp) -> iced::Task<Message> {
    app.mode = Mode::View;
    iced::Task::none()
}

pub fn handle_delete_book(_: &mut BookshelfApp, id: i32) -> iced::Task<Message> {
    iced::Task::perform(
        async move {
            match db::delete_book(id) {
                Ok(count) => Ok(count),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::BookDeleted,
    )
}

pub fn handle_books_loaded(
    app: &mut BookshelfApp,
    result: Result<Vec<BookWithAuthor>, String>,
) -> iced::Task<Message> {
    match result {
        Ok(books) => {
            app.books = books;
            app.filtered_books = None; // Reset filtered books when loading all books
            app.is_searching = false;

            // Apply sorting directly to the loaded books
            sort_books(&mut app.books, &app.sort_field, &app.sort_direction);
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    iced::Task::none()
}

pub fn handle_book_deleted(
    app: &mut BookshelfApp,
    result: Result<usize, String>,
) -> iced::Task<Message> {
    app.mode = Mode::View; // Ensure we go back to view mode

    match result {
        Ok(_) => app.update(Message::LoadBooks),
        Err(e) => {
            app.error = Some(e);
            app.update(Message::LoadBooks) // Always go back to book list even on error
        }
    }
}

// View functions for books
pub fn view(app: &BookshelfApp) -> Element<Message> {
    match &app.mode {
        Mode::View => view_book_list(app),
        Mode::Add | Mode::Edit => view_book_form(app),
        Mode::ConfirmDelete(id, title) => view_delete_confirmation(app, *id, title),
        Mode::ViewDetails => view_book_list(app),
    }
}

fn view_book_list(app: &BookshelfApp) -> Element<Message> {
    let add_button = button("Add New Book")
        .on_press(Message::AddBookMode)
        .style(button::primary);

    let books_to_display = if app.is_searching {
        app.filtered_books.as_ref().unwrap_or(&app.books)
    } else {
        &app.books
    };

    let search_status = create_search_status_label(app);

    let book_list_content = if books_to_display.is_empty() {
        create_empty_list_label(app)
    } else {
        create_books_list(books_to_display)
    };

    column![
        row![
            text(search_status).size(24),
            iced::widget::horizontal_space(),
            add_button
        ]
        .padding(15)
        .width(Length::Fill),
        scrollable(container(book_list_content).width(Length::Fill)).height(Length::Fill)
    ]
    .spacing(20)
    .padding(25)
    .into()
}

fn create_books_list(books_to_display: &Vec<BookWithAuthor>) -> Column<Message> {
    let mut list = column![].spacing(15).width(Length::Fill).padding(20);

    for book in books_to_display {
        let author_name = book
            .author
            .as_ref()
            .and_then(|a| a.Name.clone())
            .unwrap_or_else(|| "No Author".to_string());

        let price_text = book
            .book
            .price
            .map(|p| format!("{:.2}zł", p))
            .unwrap_or_else(|| "No price".to_string());

        let book_row = row![
            column![
                text(&book.book.title).size(18),
                text(format!("By: {}", author_name)).size(14),
                text(price_text).size(14),
            ]
            .spacing(8)
            .width(Length::Fill),
            button("Edit")
                .on_press(Message::EditBookMode(book.clone()))
                .style(button::secondary)
                .padding(8),
            button("Delete")
                .on_press(Message::ConfirmDeleteBook(
                    book.book.id,
                    book.book.title.clone()
                ))
                .style(button::danger)
                .padding(8),
        ]
        .spacing(15)
        .padding(10)
        .align_y(iced::Alignment::Center);

        list = list.push(
            container(book_row)
                .padding(10)
                .style(container::bordered_box),
        );
    }
    list
}

fn create_empty_list_label(app: &BookshelfApp) -> Column<Message> {
    column![text(if app.is_searching {
        format!("No books found matching '{}'", app.search_term_displayed)
    } else {
        "No books found".to_string()
    })
    .size(16)]
    .spacing(5)
    .width(Length::Fill)
    .padding(20)
}

fn create_search_status_label(app: &BookshelfApp) -> String {
    let search_status = if app.is_searching {
        if let Some(filtered) = &app.filtered_books {
            if filtered.is_empty() {
                format!("No books found matching '{}'", app.search_term_displayed)
            } else {
                format!(
                    "Found {} books matching '{}'",
                    filtered.len(),
                    app.search_term_displayed
                )
            }
        } else {
            "Search results".to_string()
        }
    } else {
        "Books".to_string()
    };
    search_status
}

fn view_book_form(app: &BookshelfApp) -> Element<Message> {
    let title = match app.mode {
        Mode::Add => "Add New Book",
        Mode::Edit => "Edit Book",
        _ => unreachable!(),
    };

    let mut author_options = app.authors.clone();
    author_options.sort_by(|a, b| a.Name.cmp(&b.Name));

    let form = column![
        text(title).size(24),
        text("Title:").size(16),
        text_input("Enter book title", &app.book_title)
            .on_input(Message::BookTitleChanged)
            .padding(10),
        text("Price:").size(16),
        text_input("Enter price (optional)", &app.book_price)
            .on_input(Message::BookPriceChanged)
            .padding(10),
        text("Bought Date (YYYY-MM-DD HH:MM:SS):").size(16),
        text_input("YYYY-MM-DD HH:MM:SS (optional)", &app.book_bought_date)
            .on_input(Message::BookBoughtDateChanged)
            .padding(10),
        text("Finished Date (YYYY-MM-DD HH:MM:SS):").size(16),
        text_input("YYYY-MM-DD HH:MM:SS (optional)", &app.book_finished_date)
            .on_input(Message::BookFinishedDateChanged)
            .padding(10),
        text("Author:").size(16),
        // Use our custom searchable dropdown instead of pick_list
        searchable_dropdown::view_author_dropdown(
            &app.author_dropdown,
            Message::ToggleAuthorDropdown,
            |term| Message::AuthorSearchChanged(term),
            |author| Message::BookAuthorSelected(author),
        ),
        row![
            button("Save")
                .on_press(Message::SaveBook)
                .style(button::primary),
            button("Cancel")
                .on_press(Message::ViewBookMode)
                .style(button::secondary),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .padding(20)
    .max_width(LIST_MAX_WIDTH);

    container(form)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

// New function to display deletion confirmation
fn view_delete_confirmation<'a>(
    _: &'a BookshelfApp,
    id: i32,
    title: &'a str,
) -> Element<'a, Message> {
    // fn view_delete_confirmation(app: &BookshelfApp, id: i32, title: &str) -> Element<Message> {
    let confirmation = column![
        text(format!("Are you sure you want to delete the book:")).size(20),
        text(format!("\"{}\"?", title)).size(24),
        text("This action cannot be undone.").size(16),
        row![
            button("Cancel")
                .on_press(Message::CancelDeleteBook)
                .style(button::secondary)
                .padding(10)
                .width(Length::Fill),
            button("Confirm Delete")
                .on_press(Message::DeleteBook(id))
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
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(container::bordered_box)
        .into()
}
