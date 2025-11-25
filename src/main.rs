// src/main.rs
use anyhow::bail;
use clap::{Parser, Subcommand};
use mdbook::preprocess::Preprocessor; // <-- this brings supports_renderer into scope
use mdbook_frontmatter_strip::{strip_frontmatter, FrontmatterStrip};
use serde_json::Value;
use std::io::{self, Read, Write};
use std::process;

#[derive(Parser)]
#[command(name = "mdbook-frontmatter-strip")]
#[command(about = "Strip YAML frontmatter from mdBook chapters")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Supports { renderer: String },
}

fn main() {
    let cli = Cli::parse();

    // FrontmatterStrip implements Default and Preprocessor
    let pre = FrontmatterStrip::default();

    match cli.command {
        Some(Command::Supports { renderer }) => {
            // supports_renderer is now in scope because we imported the trait
            let supported = Preprocessor::supports_renderer(&pre, &renderer);
            process::exit(if supported { 0 } else { 1 });
        }
        None => {
            if let Err(e) = run_preprocessor() {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    }
}

fn run_preprocessor() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let mut values: Vec<Value> = serde_json::from_str(&input)?;
    if values.len() != 2 {
        bail!("Expected [context, book] array from mdbook");
    }

    let book_obj = values[1]
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Book is not an object"))?;
    if let Some(items) = book_obj.get_mut("items").and_then(|v| v.as_array_mut()) {
        for item in items {
            process_book_item(item);
        }
    } else {
        bail!("No 'items' array found in book");
    }

    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &values[1])?;
    writeln!(stdout)?;

    Ok(())
}
fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Target chapters: strip "content" if present and a string
            if let Some(content_val) = map.get_mut("content") {
                if let Value::String(content) = content_val {
                    let (stripped, _) = strip_frontmatter(content);
                    *content_val = Value::String(stripped.trim_start_matches('\n').to_string());
                }
            }
            // Recurse on sub-items for nested sections
            if let Some(items_val) = map.get_mut("items") {
                if let Value::Array(items) = items_val {
                    for item in items.iter_mut() {
                        process_book_item(item);
                    }
                }
            }
            // No need for full map recursion; only sub-items matter
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                process_book_item(item);
            }
        }
        _ => {}
    }
}
