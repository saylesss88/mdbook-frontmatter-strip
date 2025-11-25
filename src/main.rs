use anyhow::{anyhow, bail, Result};
use mdbook::preprocess::Preprocessor;
use mdbook_frontmatter_strip::{strip_frontmatter, FrontmatterStrip};
use serde_json::Value;
use std::io::{self, Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle the `supports` subcommand
    if args.len() == 3 && args[1] == "supports" {
        let renderer = &args[2];
        let supported = FrontmatterStrip.supports_renderer(renderer);
        process::exit(if supported { 0 } else { 1 });
    }

    // Otherwise run the actual preprocessor
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let mut values: Vec<Value> =
        serde_json::from_str(&input).map_err(|e| anyhow!("Failed to parse input JSON: {e}"))?;

    if values.len() != 2 {
        bail!("Expected [context, book] array from mdbook");
    }

    let book = values
        .get_mut(1)
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| anyhow!("Second element is not a book object"))?;

    if let Some(items) = book.get_mut("items").and_then(|v| v.as_array_mut()) {
        for item in items {
            process_item(item);
        }
    }

    // Write only the modified book back
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &values[1])?;
    writeln!(stdout)?;
    Ok(())
}

fn process_item(value: &mut Value) {
    if let Value::Object(map) = value {
        if let Some(Value::String(content)) = map.get_mut("content") {
            let (stripped, _) = strip_frontmatter(content);
            *content = stripped.trim_start_matches('\n').to_string();
        }

        if let Some(Value::Array(subitems)) = map.get_mut("items") {
            for item in subitems {
                process_item(item);
            }
        }
    }
}
