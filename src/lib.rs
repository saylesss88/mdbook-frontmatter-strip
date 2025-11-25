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
    let mut lines = input.lines().peekable();

    if lines.peek().map(|l| l.trim()) != Some("---") {
        return (input.to_owned(), None);
    }

    let mut frontmatter_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    lines.next(); // skip first ---

    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            frontmatter_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    let body = body_lines.join("\n").trim().to_owned();
    let frontmatter = (!frontmatter_lines.is_empty()).then(|| {
        let fm = frontmatter_lines.join("\n").trim_end().to_owned();
        format!("---\n{fm}\n---")
    });

    (body, frontmatter)
}
