//! Strip YAML frontmatter from Markdown chapter content in an mdbook book.
//!
//! The public API has two layers:
//!
//! - **String level:** [`parse_frontmatter`] and [`strip_frontmatter`] work
//!   on any Markdown `&str`, with `_with` variants that take [`ParseOptions`].
//! - **mdBook level:** [`process_book_item`] walks an mdBook `BookItem` JSON
//!   value and strips every chapter in place.

#![deny(missing_docs)]

mod book;
/// The actual stripping code
mod frontmatter;

pub use book::{process_book_item, process_book_item_with};
pub use frontmatter::yaml_frontmatter::{
    Frontmatter, ParseOptions, parse_frontmatter, parse_frontmatter_with, strip_frontmatter,
    strip_frontmatter_with,
};
