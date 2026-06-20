//! Strip YAML frontmatter from Markdown chapter content in an mdbook book.
//!
//! - [`frontmatter`] contains the pure string-level stripping logic.
//! - [`book`] walks the mdbook `BookItem` JSON tree and applies it.

mod book;
mod frontmatter;

pub use book::process_book_item;
