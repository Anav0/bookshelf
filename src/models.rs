// src/models.rs
use crate::schema::{Author, Books};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

pub type ID = i32;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = Author)]
#[diesel(primary_key(Id))]
pub struct AuthorModel {
    pub Id: ID,
    pub Name: Option<String>,
}

impl Eq for AuthorModel {}
impl PartialEq for AuthorModel {
    fn eq(&self, other: &Self) -> bool {
        self.Id == other.Id
    }
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = Author)]
pub struct NewAuthor {
    pub Name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = Books)]
pub struct BookModel {
    pub id: ID,
    pub title: String,
    pub price: Option<f32>,
    pub bought: Option<NaiveDateTime>,
    pub finished: Option<NaiveDateTime>,
    pub added: Option<NaiveDateTime>,
    pub AuthorFK: Option<ID>,
}

impl Eq for BookModel {}
impl PartialEq for BookModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = Books)]
pub struct NewBook {
    pub title: String,
    pub price: Option<f32>,
    pub bought: Option<NaiveDateTime>,
    pub finished: Option<NaiveDateTime>,
    pub added: Option<NaiveDateTime>,
    pub AuthorFK: Option<ID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookWithAuthor {
    pub book: BookModel,
    pub author: Option<AuthorModel>,
}

// Implement Display for AuthorModel for use in the pick_list
impl std::fmt::Display for AuthorModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.Name
                .clone()
                .unwrap_or_else(|| "Unnamed Author".to_string())
        )
    }
}
