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

    let book = &mut values[1];
    process_book(book);

    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, book)?;
    writeln!(stdout)?; // mdbook requires the trailing newline

    Ok(())
}

fn process_book(value: &mut Value) {
    if let Value::Array(items) = value {
        for item in items {
            process_book_item(item);
        }
    }
}

fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                if let Some(Value::String(content)) = chapter.get_mut("content") {
                    let (stripped, _) = strip_frontmatter(content);
                    *content = stripped.trim_start_matches('\n').to_owned();
                }
            }
            for (_key, val) in map.iter_mut() {
                process_book_item(val);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                process_book_item(item);
            }
        }
        _ => {}
    }
}
