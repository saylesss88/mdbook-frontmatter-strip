use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct FrontmatterStrip; // Unit struct; no Default needed

impl Preprocessor for FrontmatterStrip {
    fn name(&self) -> &str {
        "frontmatter-strip"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                let (stripped, _) = strip_frontmatter(&ch.content);
                ch.content = stripped
                    .trim_start_matches('\n')
                    .trim_end_matches('\n')
                    .to_owned()
                    + "\n";
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

pub fn strip_frontmatter(input: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return (input.to_owned(), None);
    }

    // Find start '---'
    let start_idx = match lines.iter().position(|l| l.trim() == "---") {
        Some(i) => i,
        None => return (input.to_owned(), None),
    };

    // Find ending '---'
    let end_idx = match lines
        .iter()
        .skip(start_idx + 1)
        .position(|l| l.trim() == "---")
    {
        Some(i) => start_idx + 1 + i,
        None => return (input.to_owned(), None), // malformed → leave content unchanged
    };

    // Extract frontmatter (inclusive of both --- lines)
    let front = lines[start_idx..=end_idx].join("\n");

    // Extract body (everything after the end delimiter)
    let body = lines[end_idx + 1..].join("\n");

    let body = body.trim().to_owned();

    (body, Some(front))
}
