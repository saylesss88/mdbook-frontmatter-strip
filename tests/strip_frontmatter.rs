use mdbook_frontmatter_strip::process_book_item;
use serde_json::json;

#[test]
fn fenced_frontmatter_is_stripped() {
    let content = "\
---
title: Post
tags: [rust]
---

# Heading

Body text.
";

    let mut chapter = json!({
        "Chapter": {
            "name": "ch",
            "content": content,
            "sub_items": []
        }
    });

    process_book_item(&mut chapter);

    let out = chapter["Chapter"]["content"].as_str().unwrap();
    assert!(out.starts_with("# Heading"), "body should start at heading");
    assert!(!out.contains("title: Post"), "frontmatter should be gone");
    assert!(out.ends_with('\n'), "should preserve trailing newline");
}

#[test]
fn unfenced_single_line_is_not_treated_as_frontmatter() {
    let content = "\
Title: This is not frontmatter

Real content here.
";

    let mut chapter = json!({
        "Chapter": {
            "name": "ch",
            "content": content,
            "sub_items": []
        }
    });

    process_book_item(&mut chapter);

    let out = chapter["Chapter"]["content"].as_str().unwrap();
    assert_eq!(out, content, "single key: value line should be preserved");
}

#[test]
fn unfenced_multi_line_yaml_is_stripped() {
    let content = "\
title: Post
date: 2025-01-01
tags: [rust]

# Heading
Body.
";

    let mut chapter = json!({
        "Chapter": {
            "name": "ch",
            "content": content,
            "sub_items": []
        }
    });

    process_book_item(&mut chapter);

    let out = chapter["Chapter"]["content"].as_str().unwrap();
    assert!(
        out.starts_with("# Heading"),
        "body should start after YAML block"
    );
    assert!(!out.contains("title:"), "yaml lines should be removed");
}

#[test]
fn url_at_top_is_not_stripped() {
    let content = "\
http://example.com

Body text.
";

    let mut chapter = json!({
        "Chapter": {
            "name": "ch",
            "content": content,
            "sub_items": []
        }
    });

    process_book_item(&mut chapter);

    let out = chapter["Chapter"]["content"].as_str().unwrap();
    assert_eq!(
        out, content,
        "URL line should not be considered frontmatter"
    );
}

#[test]
fn nested_sub_items_are_processed() {
    let content = "\
---
title: Nested
---

# Nested
";

    let mut book = json!({
        "Part": {
            "name": "p",
            "sections": [
                {
                    "Chapter": {
                        "name": "outer",
                        "content": content,
                        "sub_items": [
                            {
                                "Chapter": {
                                    "name": "inner",
                                    "content": content,
                                    "sub_items": []
                                }
                            }
                        ]
                    }
                }
            ]
        }
    });

    process_book_item(&mut book);

    let outer = book["Part"]["sections"][0]["Chapter"]["content"]
        .as_str()
        .unwrap();
    let inner = book["Part"]["sections"][0]["Chapter"]["sub_items"][0]["Chapter"]["content"]
        .as_str()
        .unwrap();

    assert!(outer.starts_with("# Nested"));
    assert!(inner.starts_with("# Nested"));
}
