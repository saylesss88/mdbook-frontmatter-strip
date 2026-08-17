//! Strip YAML frontmatter from Markdown chapter content in an mdbook book.
//!
//! Internally split into a `frontmatter` module (pure string-level stripping
//! logic) and a `book` module (walks the mdbook `BookItem` JSON tree and
//! applies it). Only [`process_book_item`] is exposed publicly.

#![deny(missing_docs)]

mod book;
/// The actual stripping code
pub mod frontmatter;

pub use book::process_book_item;
