#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use mdbook_frontmatter_strip::frontmatter::strip_frontmatter;

#[derive(Arbitrary, Debug)]
struct FuzzDoc {
    /// Simulates leading blank lines before frontmatter
    leading_newlines: u8,
    fence_style: FenceStyle,
    kv_pairs: Vec<(String, String)>,
    body: String,
}

#[derive(Arbitrary, Debug)]
enum FenceStyle {
    Fenced { close: bool }, // with or without closing ---
    Unfenced,
    None,
}

fuzz_target!(|doc: FuzzDoc| {
    let leading = "\n".repeat((doc.leading_newlines % 5) as usize);
    let input = match doc.fence_style {
        FenceStyle::Fenced { close } => {
            let kvs: String = doc
                .kv_pairs
                .iter()
                .map(|(k, v)| format!("{}: {}\n", k, v))
                .collect();
            let closing = if close { "---\n" } else { "" };
            format!("{}---\n{}{}{}", leading, kvs, closing, doc.body)
        }
        FenceStyle::Unfenced => {
            let kvs: String = doc
                .kv_pairs
                .iter()
                .map(|(k, v)| format!("{}: {}\n", k, v))
                .collect();
            format!("{}{}\n{}", leading, kvs, doc.body)
        }
        FenceStyle::None => {
            format!("{}{}", leading, doc.body)
        }
    };
    let _ = strip_frontmatter(&input);
});
