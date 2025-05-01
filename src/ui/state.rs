// src/ui/state.rs
use crate::db;
use crate::models::{AuthorModel, BookWithAuthor};
use crate::ui::{
    author_view, book_view, sort_books, Message, Mode, SortDirection, SortField, Tab,
};
use iced::{executor, Application, Command, Element, Theme};

/// Main application state struct
pub struct BookshelfApp {
    // State
    pub current_tab: Tab,
    pub mode: Mode,

    // Sorting state
    pub sort_field: SortField,
    pub sort_direction: SortDirection,

    // Search state
    pub search_query: String,
    pub search_term_displayed: String, // Static term that was searched for
    pub is_searching: bool,
    pub filtered_books: Option<Vec<BookWithAuthor>>,

    // Book state
    pub books: Vec<BookWithAuthor>,
    pub current_book: Option<BookWithAuthor>,
    pub book_title: String,
    pub book_price: String,
    pub book_bought_date: String,
    pub book_finished_date: String,
    pub selected_author: Option<AuthorModel>,

    // Author state
    pub authors: Vec<AuthorModel>,
    pub current_author: Option<AuthorModel>,
    pub author_name: String,
    pub author_books: Vec<BookWithAuthor>,  // Books by the current author

    // Error handling
    pub error: Option<String>,
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
                author_books: Vec::new(),
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

            // Sorting messages
            Message::SortFieldSelected(field) => {
                self.sort_field = field;
                Command::none()
            }

            Message::SortDirectionSelected(direction) => {
                self.sort_direction = direction;
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

                    // Apply current sorting to search results
                    return self.update(Message::ApplySorting);
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
            Message::ViewAuthorDetails(author) => author_view::handle_view_author_details(self, author),
            Message::AuthorBooksLoaded(result) => author_view::handle_author_books_loaded(self, result),
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
        crate::ui::common::view(self)
    }
}