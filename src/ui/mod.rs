// src/ui/mod.rs
mod author_view;
mod book_view;
mod common;
mod messages;
mod state;
mod utils;
mod variables;

// Re-exports for other modules
pub use author_view::*;
pub use book_view::*;
pub use common::*;
pub use messages::*;
pub use state::*;
pub use utils::*;
pub use variables::*;

// Export BookshelfApp for main.rs
pub use state::BookshelfApp;