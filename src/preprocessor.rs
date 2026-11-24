//! Implements the mdBook preprocessor protocol: read `[context, book]` JSON
//! from stdin, strip frontmatter from every chapter, write the modified book
//! JSON back to stdout.

use std::{
    error, fmt,
    io::{self, Read, Write},
};

use serde_json::Value;

use mdbook_frontmatter_strip::ParseOptions;

/// The `[preprocessor.<NAME>]` table this preprocessor reads its options from.
const NAME: &str = "frontmatter-strip";

/// Everything that can go wrong while running as an mdBook preprocessor.
///
/// Frontmatter stripping itself is infallible, so every variant here is about
/// the mdBook protocol: reading stdin, parsing/writing JSON, and the shape of
/// what mdBook sent.
#[derive(Debug)]
pub enum Error {
    /// Reading stdin or writing stdout failed.
    Io(io::Error),
    /// stdin wasn't valid JSON, or the book couldn't be serialized.
    Json(serde_json::Error),
    /// Valid JSON, but not the `[context, book]` shape mdBook sends.
    MalformedInput(String),
}

/// Shorthand for results in this module.
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::MalformedInput(msg) => write!(f, "malformed input: {msg}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::MalformedInput(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Run the full preprocessor: read stdin, process, write stdout.
pub fn run(mut input: impl Read, output: impl Write) -> Result<()> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let mut values = parse_input(&buf)?;
    let opts = parse_options(&values[0])?;
    process_book(&mut values[1], opts)?;
    write_output(output, &values[1])
}

/// Convenience wrapper around [`run`] using real stdin/stdout.
pub fn run_with_stdio() -> Result<()> {
    run(io::stdin(), io::stdout().lock())
}

/// Parse the `[context, book]` array mdBook sends on stdin.
fn parse_input(input: &str) -> Result<Vec<Value>> {
    let values: Vec<Value> = serde_json::from_str(input)?;
    if values.len() != 2 {
        return Err(Error::MalformedInput(format!(
            "Expected [context, book] array from mdBook (got len = {})",
            values.len()
        )));
    }
    Ok(values)
}

/// Read `[preprocessor.frontmatter-strip]` options from the mdBook context.
///
/// A missing table or key means the default. A key with the wrong type is
/// an error, so a typo like `unfenced = "true"` doesn't silently do nothing.
fn parse_options(ctx: &Value) -> Result<ParseOptions> {
    let table = ctx
        .get("config")
        .and_then(|c| c.get("preprocessor"))
        .and_then(|p| p.get(NAME));

    let unfenced = match table.and_then(|t| t.get("unfenced")) {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(other) => {
            return Err(Error::MalformedInput(format!(
                "`preprocessor.{NAME}.unfenced` must be true or false, got {other}"
            )));
        }
    };
    Ok(ParseOptions::new().unfenced(unfenced))
}

/// Strip frontmatter from every chapter in the book, in place.
///
/// mdBook 0.5 sends the chapter list as `items`; 0.4 called it `sections`.
fn process_book(book: &mut Value, opts: ParseOptions) -> Result<()> {
    let key = if book.get("items").is_some() {
        "items"
    } else {
        "sections"
    };
    let Some(Value::Array(items)) = book.get_mut(key) else {
        return Err(Error::MalformedInput(
            "book JSON has no 'items' (mdBook 0.5) or 'sections' (mdBook 0.4) array".into(),
        ));
    };
    for item in items {
        mdbook_frontmatter_strip::process_book_item_with(item, opts);
    }
    Ok(())
}

/// Write the modified book JSON (without the mdBook context) to `output`.
fn write_output(mut output: impl Write, book: &Value) -> Result<()> {
    serde_json::to_writer(&mut output, book)?;
    writeln!(output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_wrong_length_array() {
        let input = "[{}]"; // only one element, not [context, book]
        let err = parse_input(input).unwrap_err();
        assert!(err.to_string().contains("got len = 1"));
    }

    #[test]
    fn rejects_invalid_json() {
        let err = parse_input("not json").unwrap_err();
        assert!(err.to_string().contains("JSON error:"));
    }

    #[test]
    fn processes_items_key() {
        let mut book = json!({
            "items": [
                {
                    "Chapter": {
                        "content": "---\ntitle: Hi\n---\nbody\n",
                        "sub_items": []
                    }
                }
            ]
        });
        process_book(&mut book, ParseOptions::new()).unwrap();
        assert_eq!(book["items"][0]["Chapter"]["content"], "body\n");
    }

    #[test]
    fn end_to_end_strips_frontmatter_and_writes_book_only() {
        let input = json!([
            { "root": "/book" },
            {
                "items": [
                    {
                        "Chapter": {
                            "content": "---\ntitle: Hi\n---\nbody\n",
                            "sub_items": []
                        }
                    }
                ]
            }
        ])
        .to_string();

        let mut output = Vec::new();
        run(input.as_bytes(), &mut output).unwrap();

        let result: Value = serde_json::from_slice(&output).unwrap();
        // Output should be the book only (no context wrapper array).
        assert_eq!(result["items"][0]["Chapter"]["content"], "body\n");
    }

    #[test]
    fn processes_sections_key_from_mdbook_0_4() {
        let mut book = json!({
            "sections": [
                {
                    "Chapter": {
                        "content": "---\ntitle: Hi\n---\nbody\n",
                        "sub_items": []
                    }
                }
            ],
            "__non_exhaustive": null
        });
        process_book(&mut book, ParseOptions::new()).unwrap();
        assert_eq!(book["sections"][0]["Chapter"]["content"], "body\n");
        // The 0.4-only field must survive the round trip.
        assert!(book.get("__non_exhaustive").is_some());
    }

    #[test]
    fn rejects_book_with_neither_key() {
        let mut book = json!({ "chapters": [] });
        let err = process_book(&mut book, ParseOptions::new()).unwrap_err();
        assert!(err.to_string().contains("'sections'"));
    }
}
