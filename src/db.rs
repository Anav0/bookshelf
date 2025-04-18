// src/db.rs
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use anyhow::Result;
use thiserror::Error;

// Important: Use r2d2 directly, not through diesel
use r2d2;
use diesel::r2d2::ConnectionManager;

use crate::models::{AuthorModel, BookModel, BookWithAuthor, NewAuthor, NewBook};
use crate::schema::{Author, Books};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

static DB_POOL: Lazy<Mutex<Option<DbPool>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),

    #[error("Database pool not initialized")]
    PoolNotInitialized,
}

// Implementation for the standalone r2d2::Error
impl From<r2d2::Error> for DbError {
    fn from(err: r2d2::Error) -> Self {
        DbError::Connection(err.to_string())
    }
}

pub fn initialize_pool() -> Result<(), DbError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .build(manager)?;

    let mut db_pool = DB_POOL.lock().unwrap();
    *db_pool = Some(pool);
    Ok(())
}

pub fn get_connection() -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, DbError> {
    let db_pool = DB_POOL.lock().unwrap();
    match &*db_pool {
        Some(pool) => Ok(pool.get()?),
        None => Err(DbError::PoolNotInitialized),
    }
}

// Author CRUD Operations
pub fn get_authors() -> Result<Vec<AuthorModel>, DbError> {
    let mut conn = get_connection()?;
    let authors = Author::table
        .select(AuthorModel::as_select())
        .load(&mut conn)?;
    Ok(authors)
}

pub fn get_author(id: i32) -> Result<AuthorModel, DbError> {
    let mut conn = get_connection()?;
    let author = Author::table
        .find(id)
        .select(AuthorModel::as_select())
        .first(&mut conn)?;
    Ok(author)
}

pub fn create_author(new_author: &NewAuthor) -> Result<AuthorModel, DbError> {
    let mut conn = get_connection()?;
    let author = diesel::insert_into(Author::table)
        .values(new_author)
        .returning(AuthorModel::as_returning())
        .get_result(&mut conn)?;
    Ok(author)
}

pub fn update_author(id: i32, author: &NewAuthor) -> Result<AuthorModel, DbError> {
    let mut conn = get_connection()?;
    let author = diesel::update(Author::table.find(id))
        .set(author)
        .returning(AuthorModel::as_returning())
        .get_result(&mut conn)?;
    Ok(author)
}

pub fn delete_author(id: i32) -> Result<usize, DbError> {
    let mut conn = get_connection()?;
    let count = diesel::delete(Author::table.find(id))
        .execute(&mut conn)?;
    Ok(count)
}

// Book CRUD Operations
pub fn get_books() -> Result<Vec<BookWithAuthor>, DbError> {
    let mut conn = get_connection()?;
    let books = Books::table
        .select(BookModel::as_select())
        .load::<BookModel>(&mut conn)?;

    let mut books_with_authors: Vec<BookWithAuthor> = Vec::new();

    for book in books {
        let author = if let Some(author_id) = book.AuthorFK {
            match Author::table.find(author_id).select(AuthorModel::as_select()).first(&mut conn) {
                Ok(author) => Some(author),
                Err(_) => None,
            }
        } else {
            None
        };

        books_with_authors.push(BookWithAuthor { book, author });
    }

    Ok(books_with_authors)
}

pub fn get_book(id: i32) -> Result<BookWithAuthor, DbError> {
    let mut conn = get_connection()?;
    let book = Books::table
        .find(id)
        .select(BookModel::as_select())
        .first(&mut conn)?;

    let author = if let Some(author_id) = book.AuthorFK {
        match Author::table.find(author_id).select(AuthorModel::as_select()).first(&mut conn) {
            Ok(author) => Some(author),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(BookWithAuthor { book, author })
}

pub fn create_book(new_book: &NewBook) -> Result<BookModel, DbError> {
    let mut conn = get_connection()?;
    let book = diesel::insert_into(Books::table)
        .values(new_book)
        .returning(BookModel::as_returning())
        .get_result(&mut conn)?;
    Ok(book)
}

pub fn update_book(id: i32, book: &NewBook) -> Result<BookModel, DbError> {
    let mut conn = get_connection()?;
    let book = diesel::update(Books::table.find(id))
        .set(book)
        .returning(BookModel::as_returning())
        .get_result(&mut conn)?;
    Ok(book)
}

pub fn delete_book(id: i32) -> Result<usize, DbError> {
    let mut conn = get_connection()?;
    let count = diesel::delete(Books::table.find(id))
        .execute(&mut conn)?;
    Ok(count)
}
