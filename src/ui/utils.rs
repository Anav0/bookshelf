// src/ui/utils.rs
use crate::models::BookWithAuthor;
use crate::ui::{SortDirection, SortField};
use std::cmp::Ordering;

/// Helper function to sort books based on given field and direction
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