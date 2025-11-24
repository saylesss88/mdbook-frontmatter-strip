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
    let mut front = String::new();
    let mut body = String::new();
    let mut in_frontmatter = false;
    let mut saw_start = false;

    for (i, line) in input.lines().enumerate() {
        let trimmed = line.trim();

        if i == 0 && trimmed == "---" {
            saw_start = true;
            in_frontmatter = true;
            front.push_str(line);
            front.push('\n');
            continue;
        }

        if in_frontmatter {
            front.push_str(line);
            front.push('\n');
            if trimmed == "---" {
                in_frontmatter = false;
            }
            continue;
        }

        body.push_str(line);
        body.push('\n');
    }

    if saw_start {
        (body, Some(front))
    } else {
        (input.to_string(), None)
    }
}
