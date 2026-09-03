//! Parse fenced, unfenced, and no frontmatter
//!
//! Usage: `cargo run --example parse`
use mdbook_frontmatter_strip::parse_frontmatter;

fn main() {
    let cases = [
        (
            "fenced",
            "---\ntitle: My Post\ndate: 2026-10-03\n---\n\n# Hello\n\nBody text.\n",
        ),
        ("unfenced", "title: My Post\nauthor: Tom\n\nBody text.\n"),
        (
            "none",
            "# No frontmatter here\n\nJust a regular markdown file.\n",
        ),
    ];

    for (label, content) in cases {
        println!("=== {label} ===");
        let fm = parse_frontmatter(content);
        match fm.yaml {
            Some(yaml) => println!("Frontmatter:\n{yaml}\n"),
            None => println!("No frontmatter found\n"),
        }
        println!("Body:\n{}", fm.body);
    }
}
