//! Simple example demonstrating the use of methods
//!
//! Usage: `cargo run --example fm_methods`
use mdbook_frontmatter_strip::parse_frontmatter;

fn main() {
    let with_fm = "---\ntitle: Hello\n---\n\nBody text.\n";
    let without_fm = "# Just a heading\n\nBody text.\n";

    for content in [with_fm, without_fm] {
        let fm = parse_frontmatter(content);
        println!("has frontmatter: {}", fm.has_frontmatter());
        println!("yaml or body:\n{}", fm.yaml_or_body());
        println!("---");
    }
}
