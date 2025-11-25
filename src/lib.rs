use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

#[derive(Default)]
pub struct FrontmatterStrip;

impl Preprocessor for FrontmatterStrip {
    fn name(&self) -> &str {
        "frontmatter-strip"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                let (stripped, _) = strip_frontmatter(&ch.content);
                ch.content = stripped.trim_start_matches('\n').to_owned();
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

pub fn strip_frontmatter(input: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return (input.to_string(), None);
    }

    let mut front = String::new();
    let mut body = String::new();
    let mut in_frontmatter = false;
    let mut saw_start = false;
    let mut end_found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let full_line = *line; // Preserve original for appending

        if !saw_start {
            if trimmed == "---" {
                saw_start = true;
                in_frontmatter = true;
                front.push_str(full_line);
                front.push('\n');
                continue;
            } else {
                // Before start: append to body (e.g., leading non-frontmatter content)
                body.push_str(full_line);
                body.push('\n');
                continue;
            }
        }

        if in_frontmatter {
            front.push_str(full_line);
            front.push('\n');
            if trimmed == "---" {
                end_found = true;
                // Body starts after this line
                for remaining_line in lines.iter().skip(i + 1) {
                    body.push_str(remaining_line);
                    body.push('\n');
                }
                break;
            }
        } else {
            body.push_str(full_line);
            body.push('\n');
        }
    }

    // If no end delimiter, treat everything after start as body
    if saw_start && !end_found {
        body.clear();
        let body_start_idx = lines.iter().position(|l| l.trim() == "---").unwrap_or(0) + 1;
        for line in lines.iter().skip(body_start_idx) {
            body.push_str(line);
            body.push('\n');
        }
    }

    let body_clean = body.trim().to_string();
    (
        body_clean,
        if saw_start {
            Some(front.trim().to_string())
        } else {
            None
        },
    )
}
