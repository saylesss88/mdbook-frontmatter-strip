//! Implements the mdBook preprocessor protocol: read `[context, book]` JSON
//! from stdin, strip frontmatter from every chapter, write the modified book
//! JSON back to stdout.

use anyhow::{Result, anyhow};
use mdbook_frontmatter_strip::process_book_item;
use serde_json::Value;
use std::io::{self, Read, Write};

/// Run the full preprocessor: read stdin, process, write stdout.
pub fn run(mut input: impl Read, output: impl Write) -> Result<()> {
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;

    let mut values = parse_input(&buf)?;
    process_book(&mut values[1])?;
    write_output(output, &values[1])
}

/// Convenience wrapper around [`run`] using real stdin/stdout.
pub fn run_with_stdio() -> Result<()> {
    run(io::stdin(), io::stdout().lock())
}

/// Parse the `[context, book]` array mdBook sends on stdin.
fn parse_input(input: &str) -> Result<Vec<Value>> {
    let values: Vec<Value> =
        serde_json::from_str(input).map_err(|e| anyhow!("Failed to parse input JSON: {e}"))?;
    if values.len() != 2 {
        return Err(anyhow!(
            "Expected [context, book] array from mdBook (got len = {})",
            values.len()
        ));
    }
    Ok(values)
}

/// Strip frontmatter from every chapter in the book, in place.
///
/// mdBook's main entry point is either `sections` or `items` depending on
/// version (0.5.x uses `sections`).
fn process_book(book: &mut Value) -> Result<()> {
    if let Some(Value::Array(sections)) = book.get_mut("sections") {
        for section in sections.iter_mut() {
            process_book_item(section);
        }
    } else if let Some(Value::Array(items)) = book.get_mut("items") {
        for item in items.iter_mut() {
            process_book_item(item);
        }
    } else {
        return Err(anyhow!(
            "Book JSON has no 'sections' or 'items'; cannot process"
        ));
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
        assert!(err.to_string().contains("Failed to parse input JSON"));
    }

    #[test]
    fn processes_sections_key() {
        let mut book = json!({
            "sections": [
                {
                    "Chapter": {
                        "content": "---\ntitle: Hi\n---\nbody\n",
                        "sub_items": []
                    }
                }
            ]
        });
        process_book(&mut book).unwrap();
        assert_eq!(book["sections"][0]["Chapter"]["content"], "body\n");
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
        process_book(&mut book).unwrap();
        assert_eq!(book["items"][0]["Chapter"]["content"], "body\n");
    }

    #[test]
    fn errors_when_neither_key_present() {
        let mut book = json!({});
        let err = process_book(&mut book).unwrap_err();
        assert!(err.to_string().contains("no 'sections' or 'items'"));
    }

    #[test]
    fn end_to_end_strips_frontmatter_and_writes_book_only() {
        let input = json!([
            { "root": "/book" },
            {
                "sections": [
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
        assert_eq!(result["sections"][0]["Chapter"]["content"], "body\n");
    }
}
