// src/ui/mod.rs
mod author_view;
mod book_view;
mod common;

pub use author_view::*;
pub use book_view::*;
pub use common::*;

use crate::db;
use crate::models::{AuthorModel, BookModel, BookWithAuthor};
use iced::{executor, Application, Command, Element, Theme};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Tab {
    Books,
    Authors,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tab::Books => write!(f, "Books"),
            Tab::Authors => write!(f, "Authors"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    View,
    Add,
    Edit,
    ConfirmDelete(i32, String), // ID and name of item to delete
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    TabSelected(Tab),

    // Book Messages
    LoadBooks,
    BooksLoaded(Result<Vec<BookWithAuthor>, String>),
    AddBookMode,
    EditBookMode(BookWithAuthor),
    ViewBookMode,
    BookTitleChanged(String),
    BookPriceChanged(String),
    BookBoughtDateChanged(String),
    BookFinishedDateChanged(String),
    BookAuthorSelected(AuthorModel),
    SaveBook,
    BookSaved(Result<BookModel, String>),
    ConfirmDeleteBook(i32, String), // Add confirmation step
    DeleteBook(i32),
    CancelDeleteBook,
    BookDeleted(Result<usize, String>),

    // Author Messages
    LoadAuthors,
    AuthorsLoaded(Result<Vec<AuthorModel>, String>),
    AddAuthorMode,
    EditAuthorMode(AuthorModel),
    ViewAuthorMode,
    AuthorNameChanged(String),
    SaveAuthor,
    AuthorSaved(Result<AuthorModel, String>),
    DeleteAuthor(i32),
    AuthorDeleted(Result<usize, String>),

    Initialize,
    Error(String),
}

pub struct BookshelfApp {
    // State
    current_tab: Tab,
    mode: Mode,

    // Book state
    books: Vec<BookWithAuthor>,
    current_book: Option<BookWithAuthor>,
    book_title: String,
    book_price: String,
    book_bought_date: String,
    book_finished_date: String,
    selected_author: Option<AuthorModel>,

    // Author state
    authors: Vec<AuthorModel>,
    current_author: Option<AuthorModel>,
    author_name: String,

    // Error handling
    error: Option<String>,
}

impl Application for BookshelfApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                current_tab: Tab::Books,
                mode: Mode::View,
                books: Vec::new(),
                current_book: None,
                book_title: String::new(),
                book_price: String::new(),
                book_bought_date: String::new(),
                book_finished_date: String::new(),
                selected_author: None,
                authors: Vec::new(),
                current_author: None,
                author_name: String::new(),
                error: None,
            },
            Command::perform(async {}, |_| Message::Initialize),
        )
    }

    fn title(&self) -> String {
        String::from("Bookshelf Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Initialize => {
                if let Err(e) = db::initialize_pool() {
                    self.error = Some(format!("Failed to initialize database: {}", e));
                    return Command::none();
                }

                return Command::batch(vec![
                    Command::perform(async {}, |_| Message::LoadBooks),
                    Command::perform(async {}, |_| Message::LoadAuthors),
                ]);
            }

            Message::TabSelected(tab) => {
                self.current_tab = tab.clone();
                self.mode = Mode::View;

                match tab {
                    Tab::Books => return self.update(Message::LoadBooks),
                    Tab::Authors => return self.update(Message::LoadAuthors),
                }
            }

            // Book messages handled in the book module
            Message::LoadBooks => book_view::handle_load_books(self),
            Message::BooksLoaded(result) => book_view::handle_books_loaded(self, result),
            Message::AddBookMode => book_view::handle_add_book_mode(self),
            Message::EditBookMode(book) => book_view::handle_edit_book_mode(self, book),
            Message::ViewBookMode => book_view::handle_view_book_mode(self),
            Message::BookTitleChanged(value) => book_view::handle_book_title_changed(self, value),
            Message::BookPriceChanged(value) => book_view::handle_book_price_changed(self, value),
            Message::BookBoughtDateChanged(value) => {
                book_view::handle_book_bought_date_changed(self, value)
            }
            Message::BookFinishedDateChanged(value) => {
                book_view::handle_book_finished_date_changed(self, value)
            }
            Message::BookAuthorSelected(author) => {
                book_view::handle_book_author_selected(self, author)
            }
            Message::SaveBook => book_view::handle_save_book(self),
            Message::BookSaved(result) => book_view::handle_book_saved(self, result),
            Message::ConfirmDeleteBook(id, title) => {
                book_view::handle_confirm_delete_book(self, id, title)
            }
            Message::CancelDeleteBook => book_view::handle_cancel_delete_book(self),
            Message::DeleteBook(id) => book_view::handle_delete_book(self, id),
            Message::BookDeleted(result) => book_view::handle_book_deleted(self, result),

            // Author messages handled in the author module
            Message::LoadAuthors => author_view::handle_load_authors(self),
            Message::AuthorsLoaded(result) => author_view::handle_authors_loaded(self, result),
            Message::AddAuthorMode => author_view::handle_add_author_mode(self),
            Message::EditAuthorMode(author) => author_view::handle_edit_author_mode(self, author),
            Message::ViewAuthorMode => author_view::handle_view_author_mode(self),
            Message::AuthorNameChanged(value) => {
                author_view::handle_author_name_changed(self, value)
            }
            Message::SaveAuthor => author_view::handle_save_author(self),
            Message::AuthorSaved(result) => author_view::handle_author_saved(self, result),
            Message::DeleteAuthor(id) => author_view::handle_delete_author(self, id),
            Message::AuthorDeleted(result) => author_view::handle_author_deleted(self, result),

            Message::Error(error) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        common::view(self)
    }
}
