// src/ui/messages.rs (additions for searchable dropdown)
use crate::models::{AuthorModel, BookModel, BookWithAuthor, ID};
use std::fmt;

/// Defines all the possible messages that can be sent in the application
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
    ConfirmDeleteBook(ID, String), // Add confirmation step
    DeleteBook(ID),
    CancelDeleteBook,
    BookDeleted(Result<usize, String>),

    // Author Messages
    LoadAuthors,
    AuthorsLoaded(Result<Vec<AuthorModel>, String>),
    AddAuthorMode,
    EditAuthorMode(AuthorModel),
    ViewAuthorMode,
    ViewAuthorDetails(AuthorModel),  // New message for viewing author details
    AuthorBooksLoaded(Result<Vec<BookWithAuthor>, String>),  // New message for loaded books
    AuthorNameChanged(String),
    SaveAuthor,
    AuthorSaved(Result<AuthorModel, String>),
    ConfirmDeleteAuthor(ID, String), // New message for delete confirmation
    DeleteAuthor(ID),
    CancelDeleteAuthor, // New message for cancel deletion
    AuthorDeleted(Result<usize, String>),

    // Searchable Dropdown Messages
    ToggleAuthorDropdown,
    AuthorSearchChanged(String),

    Initialize,
    Error(String),
}

/// Defines the application display modes
#[derive(Debug, Clone)]
pub enum Mode {
    View,
    ViewDetails,  // Mode for viewing author details
    Add,
    Edit,
    ConfirmDelete(ID, String), // ID and name of item to delete
}

/// Defines the available tabs in the application
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

/// Defines the available sort fields
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

/// Defines the sort directions
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