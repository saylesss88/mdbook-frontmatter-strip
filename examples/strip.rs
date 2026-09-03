//! Demonstrates using mdbook-frontmatter-strip as a library.
//!
//! Run the example: `cargo run --example strip`
//! Usage with piped input: `echo "---\ntitle: Hi\n---\nbody" | cargo run --example strip`

use std::io::{self, IsTerminal, Read};

use mdbook_frontmatter_strip::{parse_frontmatter, strip_frontmatter};

fn main() {
    let content = if io::stdin().is_terminal() {
        // No piped input, use the demo string
        "---\ntitle: My Post\ndate: 2026-10-03\n---\n\n# Hello\n\nThis is the body.\n".to_string()
    } else {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .expect("failed to read stdin");
        input
    };

    println!("=== Body only ===");
    println!("{}", strip_frontmatter(&content));

    println!("=== Parsed ===");
    let fm = parse_frontmatter(&content);
    match fm.yaml {
        Some(yaml) => println!("Frontmatter:\n{yaml}\n"),
        None => println!("No frontmatter found\n"),
    }
    println!("Body:\n{}", fm.body);
}
