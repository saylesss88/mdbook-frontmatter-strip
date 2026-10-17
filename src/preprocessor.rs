//! Implements the mdBook preprocessor protocol: read `[context, book]` JSON
//! from stdin, strip frontmatter from every chapter, write the modified book
//! JSON back to stdout.

use std::io::{self, Read, Write};

use serde_json::Value;

use mdbook_frontmatter_strip::ParseOptions;
use mdbook_frontmatter_strip::error::{Error, Result};

/// The `[preprocessor.<NAME>]` table this preprocessor reads its options from.
const NAME: &str = "frontmatter-strip";

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
    let values: Vec<Value> = serde_json::from_str(input).map_err(Error::Json)?;
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
fn process_book(book: &mut Value, opts: ParseOptions) -> Result<()> {
    let Some(Value::Array(items)) = book.get_mut("items") else {
        return Err(Error::MalformedInput("book JSON has no 'items'".into()));
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
}
