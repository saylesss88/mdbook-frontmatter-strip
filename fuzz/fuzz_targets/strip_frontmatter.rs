#![no_main]

use libfuzzer_sys::fuzz_target;
use mdbook_frontmatter_strip::frontmatter::strip_frontmatter;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = strip_frontmatter(s);
    }
});
