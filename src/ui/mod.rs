// src/ui/mod.rs
mod author_view;
mod book_view;
mod common;
mod messages;
mod state;
mod utils;
mod variables;

// Add our new component module
pub mod components {
    pub mod searchable_dropdown;
}

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

// Define common constants
pub const LIST_MAX_WIDTH: f32 = 600.0;
