//! Walk an mdbook `BookItem` JSON tree and strip frontmatter from chapter
//! content in place.

use crate::frontmatter::strip_frontmatter;
use serde_json::{Map, Value};

/// Recursively walk an mdbook `BookItem`-shaped [`Value`], stripping
/// frontmatter from every `Chapter`'s content.
///
/// Handles the `Chapter` and `Part` variants explicitly (mdbook's
/// `BookItem` enum), plus a few common container keys (`items`, `sub_items`)
/// so the walk degrades gracefully across mdbook JSON-shape variations
pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                strip_chapter_content(chapter);
                // Recurse into sub_items
                if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
                    for child in sub_items {
                        process_book_item(child);
                    }
                }
            }
            if let Some(Value::Object(part)) = map.get_mut("Part") {
                for key in &["items", "sub_items"] {
                    if let Some(Value::Array(children)) = part.get_mut(*key) {
                        for child in children {
                            process_book_item(child);
                        }
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                process_book_item(item);
            }
        }
        _ => {}
    }
}

/// Strip frontmatter from a chapter's `content` field.
fn strip_chapter_content(chapter: &mut Map<String, Value>) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let stripped = strip_frontmatter(content);
        if stripped != content {
            *content = stripped.to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strips_frontmatter_from_chapter_content() {
        let mut value = json!({
            "Chapter": {
                "content": "---\ntitle: Hi\n---\nbody\n",
                "sub_items": []
            }
        });
        process_book_item(&mut value);
        assert_eq!(value["Chapter"]["content"], "body\n");
    }

    #[test]
    fn recurses_into_sub_items() {
        let mut value = json!({
            "Chapter": {
                "content": "no frontmatter\n",
                "sub_items": [
                    {
                        "Chapter": {
                            "content": "---\ntitle: Nested\n---\nnested body\n",
                            "sub_items": []
                        }
                    }
                ]
            }
        });
        process_book_item(&mut value);
        assert_eq!(
            value["Chapter"]["sub_items"][0]["Chapter"]["content"],
            "nested body\n"
        );
    }

    #[test]
    fn recurses_into_part_items() {
        let mut value = json!({
            "Part": {
                "items": [
                    {
                        "Chapter": {
                            "content": "---\ntitle: In Part\n---\npart body\n",
                            "sub_items": []
                        }
                    }
                ]
            }
        });
        process_book_item(&mut value);
        assert_eq!(
            value["Part"]["items"][0]["Chapter"]["content"],
            "part body\n"
        );
    }

    #[test]
    fn recurses_into_top_level_array() {
        let mut value = json!([
            {
                "Chapter": {
                    "content": "---\ntitle: A\n---\nbody a\n",
                    "sub_items": []
                }
            }
        ]);
        process_book_item(&mut value);
        assert_eq!(value[0]["Chapter"]["content"], "body a\n");
    }

    #[test]
    fn chapter_without_content_does_not_panic() {
        let mut value = json!({
            "Chapter": {
                "sub_items": []
            }
        });
        process_book_item(&mut value); // should not panic
    }
}
