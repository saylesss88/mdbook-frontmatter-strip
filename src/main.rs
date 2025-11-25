use anyhow::{anyhow, Result};
use mdbook_frontmatter_strip::process_book_item;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::process;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _bin = args.next(); // program name

    match args.next().as_deref() {
        // mdBook capability probe: `mdbook-frontmatter-strip supports <renderer>`
        Some("supports") => {
            let renderer = args.next().unwrap_or_default();
            if renderer == "html" {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }

        // Manual CLI: `--version` or `-V`
        Some("--version") | Some("-V") => {
            print_version();
            Ok(())
        }

        // No args -> normal mdBook preprocessing
        None => run_preprocessor(),

        // Anything else: print a simple usage message and fail
        Some(other) => {
            eprintln!("Unknown argument: {other}");
            eprintln!("Usage:");
            eprintln!("  mdbook-frontmatter-strip                     # mdBook preprocessor");
            eprintln!("  mdbook-frontmatter-strip supports <renderer>");
            eprintln!("  mdbook-frontmatter-strip --version");
            process::exit(1);
        }
    }
}

fn print_version() {
    // Uses Cargo’s built-in metadata from Cargo.toml. [web:13][web:16]
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

fn run_preprocessor() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // mdBook passes [context, book]
    let mut values: Vec<Value> =
        serde_json::from_str(&input).map_err(|e| anyhow!("Failed to parse input JSON: {e}"))?;

    if values.len() != 2 {
        return Err(anyhow!(
            "Expected [context, book] array from mdBook (got len = {})",
            values.len()
        ));
    }

    let book = &mut values[1];

    // mdBook 0.5.x main entry is sections or items. [web:1]
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

    // Output modified book JSON (without context)
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &values[1])?;
    writeln!(stdout)?;
    Ok(())
}
