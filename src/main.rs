use anyhow::{anyhow, Result};
use mdbook_frontmatter_strip::process_book_item;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::process;

fn main() -> Result<()> {
    // Handle `supports` subcommand: `mdbook-frontmatter-strip supports html`
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "supports" {
        let renderer = &args[2];
        // We only care about HTML builds
        if renderer == "html" {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    // Otherwise, run as a normal preprocessor
    run_preprocessor()
}

fn run_preprocessor() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    // mdBook passes [context, book]
    let mut values: Vec<Value> =
        serde_json::from_str(&input).map_err(|e| anyhow!("Failed to parse input JSON: {e}"))?;

    if values.len() != 2 {
        return Err(anyhow!("Expected [context, book] array from mdBook"));
    }

    let book = &mut values[1];

    // mdBook 0.5.x uses `sections` as the main entry; fall back to `items` just in case. [web:59]
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
            "Book JSON has no 'sections' or 'items' array; cannot process"
        ));
    }

    // Output modified book (just the book value, not [context, book])
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &values[1])?;
    writeln!(stdout)?;
    Ok(())
}
