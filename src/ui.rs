// src/ui.rs
use crate::db;
use crate::models::{AuthorModel, BookModel, BookWithAuthor, NewAuthor, NewBook};
use chrono::{Local, NaiveDateTime};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{executor, Application, Color, Command, Element, Length, Theme};
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
    DeleteBook(i32),
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

            // Books
            Message::LoadBooks => {
                return Command::perform(
                    async {
                        match db::get_books() {
                            Ok(books) => Ok(books),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::BooksLoaded,
                );
            }

            Message::BooksLoaded(result) => {
                match result {
                    Ok(books) => {
                        self.books = books;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }

            Message::AddBookMode => {
                self.mode = Mode::Add;
                self.current_book = None;
                self.book_title = String::new();
                self.book_price = String::new();
                self.book_bought_date = String::new();
                self.book_finished_date = String::new();
                self.selected_author = None;

                return Command::perform(async {}, |_| Message::LoadAuthors);
            }

            Message::EditBookMode(book) => {
                self.mode = Mode::Edit;
                self.current_book = Some(book.clone());
                self.book_title = book.book.title;
                self.book_price = book.book.price.map_or_else(String::new, |p| p.to_string());
                self.book_bought_date = book.book.bought.map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string());
                self.book_finished_date = book.book.finished.map_or_else(String::new, |d| d.format("%Y-%m-%d %H:%M:%S").to_string());
                self.selected_author = book.author;

                return Command::perform(async {}, |_| Message::LoadAuthors);
            }

            Message::ViewBookMode => {
                self.mode = Mode::View;
                self.current_book = None;

                return self.update(Message::LoadBooks);
            }

            Message::BookTitleChanged(value) => {
                self.book_title = value;
            }

            Message::BookPriceChanged(value) => {
                self.book_price = value;
            }

            Message::BookBoughtDateChanged(value) => {
                self.book_bought_date = value;
            }

            Message::BookFinishedDateChanged(value) => {
                self.book_finished_date = value;
            }

            Message::BookAuthorSelected(author) => {
                self.selected_author = Some(author);
            }

            Message::SaveBook => {
                let price = if self.book_price.is_empty() {
                    None
                } else {
                    match self.book_price.parse::<f32>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            self.error = Some("Invalid price format".to_string());
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

                let bought_date = parse_datetime(&self.book_bought_date);
                let finished_date = parse_datetime(&self.book_finished_date);

                let now = Local::now().naive_local();
                let added_date = self.current_book.as_ref().and_then(|b| b.book.added).unwrap_or(now);

                // Extract book_id outside the closure if we're in edit mode
                let book_id = self.current_book.as_ref().map(|book| book.book.id);

                let new_book = NewBook {
                    title: self.book_title.clone(),
                    price,
                    bought: bought_date,
                    finished: finished_date,
                    added: Some(added_date),
                    AuthorFK: self.selected_author.as_ref().map(|a| a.Id),
                };

                return Command::perform(
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
                );
            }

            Message::BookSaved(result) => {
                match result {
                    Ok(_) => {
                        self.mode = Mode::View;
                        return self.update(Message::LoadBooks);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }

            Message::DeleteBook(id) => {
                return Command::perform(
                    async move {
                        match db::delete_book(id) {
                            Ok(count) => Ok(count),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::BookDeleted,
                );
            }

            Message::BookDeleted(result) => {
                match result {
                    Ok(_) => return self.update(Message::LoadBooks),
                    Err(e) => self.error = Some(e),
                }
            }

            // Authors
            Message::LoadAuthors => {
                return Command::perform(
                    async {
                        match db::get_authors() {
                            Ok(authors) => Ok(authors),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::AuthorsLoaded,
                );
            }

            Message::AuthorsLoaded(result) => {
                match result {
                    Ok(authors) => {
                        self.authors = authors;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }

            Message::AddAuthorMode => {
                self.mode = Mode::Add;
                self.current_author = None;
                self.author_name = String::new();
            }

            Message::EditAuthorMode(author) => {
                self.mode = Mode::Edit;
                self.current_author = Some(author.clone());
                self.author_name = author.Name.unwrap_or_default();
            }

            Message::ViewAuthorMode => {
                self.mode = Mode::View;
                self.current_author = None;

                return self.update(Message::LoadAuthors);
            }

            Message::AuthorNameChanged(value) => {
                self.author_name = value;
            }

            Message::SaveAuthor => {
                let new_author = NewAuthor {
                    Name: Some(self.author_name.clone()),
                };

                // Extract author_id outside the closure if we're in edit mode
                let author_id = self.current_author.as_ref().map(|author| author.Id);

                return Command::perform(
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
                );
            }

            Message::AuthorSaved(result) => {
                match result {
                    Ok(_) => {
                        self.mode = Mode::View;
                        return self.update(Message::LoadAuthors);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }

            Message::DeleteAuthor(id) => {
                return Command::perform(
                    async move {
                        match db::delete_author(id) {
                            Ok(count) => Ok(count),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::AuthorDeleted,
                );
            }

            Message::AuthorDeleted(result) => {
                match result {
                    Ok(_) => return self.update(Message::LoadAuthors),
                    Err(e) => self.error = Some(e),
                }
            }

            Message::Error(error) => {
                self.error = Some(error);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let tab_row = row![
            button(text("Books").size(20))
                .on_press(Message::TabSelected(Tab::Books))
                .style(if matches!(self.current_tab, Tab::Books) {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                }),
            button(text("Authors").size(20))
                .on_press(Message::TabSelected(Tab::Authors))
                .style(if matches!(self.current_tab, Tab::Authors) {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                }),
        ]
        .spacing(10)
        .padding(10);

        let content = match self.current_tab {
            Tab::Books => self.view_books(),
            Tab::Authors => self.view_authors(),
        };

        let error_message = if let Some(error) = &self.error {
            container(
                text(error)
                    .style(Color::from_rgb(0.8, 0.0, 0.0))
                    .size(14),
            )
            .padding(10)
            .width(Length::Fill)
        } else {
            container(text("")).width(Length::Fill)
        };

        column![
            tab_row,
            error_message,
            content,
        ]
        .into()
    }
}

impl BookshelfApp {
    fn view_books(&self) -> Element<Message> {
        match self.mode {
            Mode::View => {
                let add_button = button("Add New Book")
                    .on_press(Message::AddBookMode)
                    .style(iced::theme::Button::Primary);

                let book_list = if self.books.is_empty() {
                    column![text("No books found").size(16)]
                        .spacing(5)
                        .width(Length::Fill)
                } else {
                    let mut col = column![].spacing(10).width(Length::Fill);

                    for book in &self.books {
                        let author_name = book.author.as_ref()
                            .and_then(|a| a.Name.clone())
                            .unwrap_or_else(|| "No Author".to_string());

                        let price_text = book.book.price
                            .map(|p| format!("${:.2}", p))
                            .unwrap_or_else(|| "No price".to_string());

                        let book_row = row![
                            column![
                                text(&book.book.title).size(18),
                                text(format!("By: {}", author_name)).size(14),
                                text(price_text).size(14),
                            ].spacing(5).width(Length::Fill),
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
                    scrollable(
                        container(book_list)
                            .padding(10)
                            .width(Length::Fill)
                    ).height(Length::Fill)
                ]
                .spacing(20)
                .padding(20)
                .into()
            }

            Mode::Add | Mode::Edit => {
                let title = match self.mode {
                    Mode::Add => "Add New Book",
                    Mode::Edit => "Edit Book",
                    _ => unreachable!(),
                };

                let author_options: Vec<AuthorModel> = self.authors.clone();

                let form = column![
                    text(title).size(24),

                    text("Title:").size(16),
                    text_input("Enter book title", &self.book_title)
                        .on_input(Message::BookTitleChanged)
                        .padding(10),

                    text("Price:").size(16),
                    text_input("Enter price (optional)", &self.book_price)
                        .on_input(Message::BookPriceChanged)
                        .padding(10),

                    text("Bought Date (YYYY-MM-DD HH:MM:SS):").size(16),
                    text_input("YYYY-MM-DD HH:MM:SS (optional)", &self.book_bought_date)
                        .on_input(Message::BookBoughtDateChanged)
                        .padding(10),

                    text("Finished Date (YYYY-MM-DD HH:MM:SS):").size(16),
                    text_input("YYYY-MM-DD HH:MM:SS (optional)", &self.book_finished_date)
                        .on_input(Message::BookFinishedDateChanged)
                        .padding(10),

                    text("Author:").size(16),
                    pick_list(
                        author_options,
                        self.selected_author.clone(),
                        Message::BookAuthorSelected,
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
        }
    }

    fn view_authors(&self) -> Element<Message> {
        match self.mode {
            Mode::View => {
                let add_button = button("Add New Author")
                    .on_press(Message::AddAuthorMode)
                    .style(iced::theme::Button::Primary);

                let author_list = if self.authors.is_empty() {
                    column![text("No authors found").size(16)]
                        .spacing(5)
                        .width(Length::Fill)
                } else {
                    let mut col = column![].spacing(10).width(Length::Fill);

                    for author in &self.authors {
                        let author_name = author.Name.clone().unwrap_or_else(|| "Unnamed Author".to_string());

                        let author_row = row![
                            text(author_name).size(18).width(Length::Fill),
                            button("Edit")
                                .on_press(Message::EditAuthorMode(author.clone()))
                                .style(iced::theme::Button::Secondary),
                            button("Delete")
                                .on_press(Message::DeleteAuthor(author.Id))
                                .style(iced::theme::Button::Destructive),
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center);

                        col = col.push(author_row);
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
                    scrollable(
                        container(author_list)
                            .padding(10)
                            .width(Length::Fill)
                    ).height(Length::Fill)
                ]
                .spacing(20)
                .padding(20)
                .into()
            }

            Mode::Add | Mode::Edit => {
                let title = match self.mode {
                    Mode::Add => "Add New Author",
                    Mode::Edit => "Edit Author",
                    _ => unreachable!(),
                };

                let form = column![
                    text(title).size(24),

                    text("Name:").size(16),
                    text_input("Enter author name", &self.author_name)
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
        }
    }
}
