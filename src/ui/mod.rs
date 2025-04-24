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
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    Title,
    Author,
    Price,
    DateAdded,
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortField::Title => write!(f, "Title"),
            SortField::Author => write!(f, "Author"),
            SortField::Price => write!(f, "Price"),
            SortField::DateAdded => write!(f, "Date Added"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDirection::Ascending => write!(f, "A-Z, Low to High"),
            SortDirection::Descending => write!(f, "Z-A, High to Low"),
        }
    }
}

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

    // Sorting
    SortFieldSelected(SortField),
    SortDirectionSelected(SortDirection),
    ApplySorting,

    // Search Messages
    SearchQueryChanged(String),
    PerformSearch,
    ClearSearch,

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

    // Sorting state
    sort_field: SortField,
    sort_direction: SortDirection,

    // Search state
    search_query: String,
    search_term_displayed: String, // Static term that was searched for
    is_searching: bool,
    filtered_books: Option<Vec<BookWithAuthor>>,

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
                sort_field: SortField::Title,
                sort_direction: SortDirection::Ascending,
                search_query: String::new(),
                search_term_displayed: String::new(),
                is_searching: false,
                filtered_books: None,
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
                self.search_query = String::new();
                self.search_term_displayed = String::new();
                self.is_searching = false;
                self.filtered_books = None;

                match tab {
                    Tab::Books => return self.update(Message::LoadBooks),
                    Tab::Authors => return self.update(Message::LoadAuthors),
                }
            }

            Message::SortFieldSelected(field) => {
                self.sort_field = field;
                // Apply sorting immediately
                let books_to_sort = if self.is_searching {
                    self.filtered_books.as_mut()
                } else {
                    Some(&mut self.books)
                };

                if let Some(books) = books_to_sort {
                    sort_books(books, &self.sort_field, &self.sort_direction);
                }
                Command::none()
            }

            Message::SortDirectionSelected(direction) => {
                self.sort_direction = direction;
                // Apply sorting immediately
                let books_to_sort = if self.is_searching {
                    self.filtered_books.as_mut()
                } else {
                    Some(&mut self.books)
                };

                if let Some(books) = books_to_sort {
                    sort_books(books, &self.sort_field, &self.sort_direction);
                }
                Command::none()
            }

            Message::ApplySorting => {
                // Sort the books based on the selected field and direction
                let books_to_sort = if self.is_searching {
                    self.filtered_books.as_mut()
                } else {
                    Some(&mut self.books)
                };

                if let Some(books) = books_to_sort {
                    sort_books(books, &self.sort_field, &self.sort_direction);
                }

                Command::none()
            }

            // Search messages
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Command::none()
            }

            Message::PerformSearch => {
                if self.search_query.is_empty() {
                    self.is_searching = false;
                    self.filtered_books = None;
                    return Command::none();
                }

                self.is_searching = true;

                // Perform local search in the Books tab
                if let Tab::Books = self.current_tab {
                    let query = self.search_query.to_lowercase();
                    let filtered: Vec<BookWithAuthor> = self
                        .books
                        .iter()
                        .filter(|book| {
                            // Search by title
                            let title_match = book.book.title.to_lowercase().contains(&query);

                            // Search by author name
                            let author_match = book
                                .author
                                .as_ref()
                                .and_then(|a| a.Name.clone())
                                .map(|name| name.to_lowercase().contains(&query))
                                .unwrap_or(false);

                            // Search by price - flexible matching without rounding
                            let price_match = book.book.price.map_or(false, |price| {
                                // Try to parse the query as a number (float or integer)
                                if let Ok(query_num) = query.parse::<f32>() {
                                    // Convert the price to string to check if it contains the query
                                    let price_str = price.to_string();

                                    // Check if the price starts with the query number
                                    // (e.g., searching for "41" should match "41.99")
                                    price_str.starts_with(&query_num.to_string()) ||

                                    // Or a direct equality check for exact prices
                                    (price == query_num)
                                } else {
                                    // If query isn't a valid number, check if price string contains the query
                                    price.to_string().contains(&query)
                                }
                            });

                            title_match || author_match || price_match
                        })
                        .cloned()
                        .collect();

                    self.filtered_books = Some(filtered);
                    self.search_term_displayed = self.search_query.clone();

                    // Apply current sorting directly to search results
                    if let Some(filtered_books) = &mut self.filtered_books {
                        sort_books(filtered_books, &self.sort_field, &self.sort_direction);
                    }
                }

                Command::none()
            }

            Message::ClearSearch => {
                self.search_query = String::new();
                self.search_term_displayed = String::new();
                self.is_searching = false;
                self.filtered_books = None;
                Command::none()
            }

            // Book messages handled in the book module
            Message::LoadBooks => book_view::handle_load_books(self),
            Message::BooksLoaded(result) => {
                let command = book_view::handle_books_loaded(self, result);
                // Apply the current sorting after loading books
                if !self.books.is_empty() {
                    self.update(Message::ApplySorting);
                }
                command
            }
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

// Helper function to sort books
pub fn sort_books(books: &mut Vec<BookWithAuthor>, field: &SortField, direction: &SortDirection) {
    books.sort_by(|a, b| {
        let order = match field {
            SortField::Title => a
                .book
                .title
                .to_lowercase()
                .cmp(&b.book.title.to_lowercase()),
            SortField::Author => {
                let a_author = a
                    .author
                    .as_ref()
                    .and_then(|author| author.Name.clone())
                    .unwrap_or_else(|| String::from(""));
                let b_author = b
                    .author
                    .as_ref()
                    .and_then(|author| author.Name.clone())
                    .unwrap_or_else(|| String::from(""));
                a_author.to_lowercase().cmp(&b_author.to_lowercase())
            }
            SortField::Price => {
                let a_price = a.book.price.unwrap_or(0.0);
                let b_price = b.book.price.unwrap_or(0.0);
                a_price.partial_cmp(&b_price).unwrap_or(Ordering::Equal)
            }
            SortField::DateAdded => {
                let a_date = a.book.added;
                let b_date = b.book.added;
                match (a_date, b_date) {
                    (Some(a_d), Some(b_d)) => a_d.cmp(&b_d),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                }
            }
        };

        match direction {
            SortDirection::Ascending => order,
            SortDirection::Descending => order.reverse(),
        }
    });
}
