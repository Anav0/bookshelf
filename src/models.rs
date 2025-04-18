use crate::schema::{Author, Books};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, PartialEq)]
#[diesel(table_name = Author)]
#[diesel(primary_key(Id))]
pub struct AuthorModel {
    pub Id: i32,
    pub Name: Option<String>,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = Author)]
pub struct NewAuthor {
    pub Name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize,PartialEq)]
#[diesel(table_name = Books)]
pub struct BookModel {
    pub title: String,
    pub price: Option<f32>,
    pub bought: Option<NaiveDateTime>,
    pub finished: Option<NaiveDateTime>,
    pub added: Option<NaiveDateTime>,
    pub AuthorFK: Option<i32>,
    pub id: i32,
}

#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize,)]
#[diesel(table_name = Books)]
pub struct NewBook {
    pub title: String,
    pub price: Option<f32>,
    pub bought: Option<NaiveDateTime>,
    pub finished: Option<NaiveDateTime>,
    pub added: Option<NaiveDateTime>,
    pub AuthorFK: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookWithAuthor {
    pub book: BookModel,
    pub author: Option<AuthorModel>,
}
