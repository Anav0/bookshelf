// src/ui/book_view.rs
use crate::db;
use crate::models::{AuthorModel, BookModel, BookWithAuthor, NewBook};
use crate::ui::{BookshelfApp, Message, Mode};
use chrono::{Local, NaiveDateTime};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length};

// Handler functions for book-related messages
pub fn handle_load_books(app: &mut BookshelfApp) -> Command<Message> {
    Command::perform(
        async {
            match db::get_books() {
                Ok(books) => Ok(books),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::BooksLoaded,
    )
}

pub fn handle_books_loaded(
    app: &mut BookshelfApp,
    result: Result<Vec<BookWithAuthor>, String>,
) -> Command<Message> {
    match result {
        Ok(books) => {
            app.books = books;
        }
        Err(e) => {
            app.error = Some(e);
        }
    }
    Command::none()
}

pub fn handle_add_book_mode(app: &mut BookshelfApp) -> Command<Message> {
    app.mode = Mode::Add;
    app.current_book = None;
    app.book_title = String::new();
    app.book_price = String::new();
    app.book_bought_date = String::new();
    app.book_finished_date = String::new();
    app.selected_author = None;

    Command::perform(async {}, |_| Message::LoadAuthors)
}

pub fn handle_edit_book_mode(app: &mut BookshelfApp, book: BookWithAuthor) -> Command<Message> {
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

    Command::perform(async {}, |_| Message::LoadAuthors)
}

pub fn handle_view_book_mode(app: &mut BookshelfApp) -> Command<Message> {
    app.mode = Mode::View;
    app.current_book = None;

    app.update(Message::LoadBooks)
}

pub fn handle_book_title_changed(app: &mut BookshelfApp, value: String) -> Command<Message> {
    app.book_title = value;
    Command::none()
}

pub fn handle_book_price_changed(app: &mut BookshelfApp, value: String) -> Command<Message> {
    app.book_price = value;
    Command::none()
}

pub fn handle_book_bought_date_changed(app: &mut BookshelfApp, value: String) -> Command<Message> {
    app.book_bought_date = value;
    Command::none()
}

pub fn handle_book_finished_date_changed(
    app: &mut BookshelfApp,
    value: String,
) -> Command<Message> {
    app.book_finished_date = value;
    Command::none()
}

pub fn handle_book_author_selected(
    app: &mut BookshelfApp,
    author: AuthorModel,
) -> Command<Message> {
    app.selected_author = Some(author);
    Command::none()
}

pub fn handle_save_book(app: &mut BookshelfApp) -> Command<Message> {
    let price = if app.book_price.is_empty() {
        None
    } else {
        match app.book_price.parse::<f32>() {
            Ok(p) => Some(p),
            Err(_) => {
                app.error = Some("Invalid price format".to_string());
                return Command::none();
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

    Command::perform(
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
) -> Command<Message> {
    match result {
        Ok(_) => {
            app.mode = Mode::View;
            app.update(Message::LoadBooks)
        }
        Err(e) => {
            app.error = Some(e);
            Command::none()
        }
    }
}

pub fn handle_delete_book(app: &mut BookshelfApp, id: i32) -> Command<Message> {
    Command::perform(
        async move {
            match db::delete_book(id) {
                Ok(count) => Ok(count),
                Err(e) => Err(e.to_string()),
            }
        },
        Message::BookDeleted,
    )
}

pub fn handle_book_deleted(
    app: &mut BookshelfApp,
    result: Result<usize, String>,
) -> Command<Message> {
    match result {
        Ok(_) => app.update(Message::LoadBooks),
        Err(e) => {
            app.error = Some(e);
            Command::none()
        }
    }
}

// View functions for books
pub fn view(app: &BookshelfApp) -> Element<Message> {
    match app.mode {
        Mode::View => view_book_list(app),
        Mode::Add | Mode::Edit => view_book_form(app),
    }
}

fn view_book_list(app: &BookshelfApp) -> Element<Message> {
    let add_button = button("Add New Book")
        .on_press(Message::AddBookMode)
        .style(iced::theme::Button::Primary);

    let book_list = if app.books.is_empty() {
        column![text("No books found").size(16)]
            .spacing(5)
            .width(Length::Fill)
    } else {
        let mut col = column![].spacing(10).width(Length::Fill);

        for book in &app.books {
            let author_name = book
                .author
                .as_ref()
                .and_then(|a| a.Name.clone())
                .unwrap_or_else(|| "No Author".to_string());

            let price_text = book
                .book
                .price
                .map(|p| format!("${:.2}", p))
                .unwrap_or_else(|| "No price".to_string());

            let book_row = row![
                column![
                    text(&book.book.title).size(18),
                    text(format!("By: {}", author_name)).size(14),
                    text(price_text).size(14),
                ]
                .spacing(5)
                .width(Length::Fill),
                button("Edit")
                    .on_press(Message::EditBookMode(book.clone()))
                    .style(iced::theme::Button::Secondary),
                button("Delete")
                    .on_press(Message::DeleteBook(book.book.id))
                    .style(iced::theme::Button::Destructive),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center);

            col = col.push(book_row);
        }

        col
    };

    column![
        row![
            text("Books").size(24),
            iced::widget::horizontal_space(Length::Fill),
            add_button
        ]
        .padding(10)
        .width(Length::Fill),
        scrollable(container(book_list).padding(10).width(Length::Fill)).height(Length::Fill)
    ]
    .spacing(20)
    .padding(20)
    .into()
}

fn view_book_form(app: &BookshelfApp) -> Element<Message> {
    let title = match app.mode {
        Mode::Add => "Add New Book",
        Mode::Edit => "Edit Book",
        _ => unreachable!(),
    };

    let author_options: Vec<AuthorModel> = app.authors.clone();

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
        pick_list(
            author_options,
            app.selected_author.clone(),
            Message::BookAuthorSelected
        )
        .placeholder("Select an author (optional)")
        .padding(10),
        row![
            button("Save")
                .on_press(Message::SaveBook)
                .style(iced::theme::Button::Primary),
            button("Cancel")
                .on_press(Message::ViewBookMode)
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
