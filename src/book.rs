//! Walk an mdbook `BookItem` JSON tree and strip frontmatter from chapter
//! content in place.

use crate::{ParseOptions, strip_frontmatter_with};
use serde_json::{Map, Value};

/// Recursively walk an mdbook `BookItem`-shaped [`Value`], stripping fenced
/// frontmatter from every `Chapter`'s content.
///
/// Equivalent to [`process_book_item_with`] with default [`ParseOptions`].
pub fn process_book_item(value: &mut Value) {
    process_book_item_with(value, ParseOptions::new());
}

/// Recursively walk an mdbook `BookItem`-shaped [`Value`], stripping
/// frontmatter from every `Chapter`'s content.
///
/// mdBook serializes `BookItem` with serde's default (externally tagged)
/// representation, so an item is one of:
///
/// - `{"Chapter": {..., "content": "...", "sub_items": [...]}}`
/// - `"Separator"`
/// - `{"PartTitle": "..."}`
///
/// Only chapters carry content. Separators and part titles are left alone,
/// and chapters under a part title are ordinary siblings in the same list,
/// so they need no special handling. An array of items is also accepted.
pub fn process_book_item_with(value: &mut Value, opts: ParseOptions) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                strip_chapter_content(chapter, opts);
                if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
                    for child in sub_items {
                        process_book_item_with(child, opts);
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                process_book_item_with(item, opts);
            }
        }
        _ => {}
    }
}

/// Strip frontmatter from a chapter's `content` field.
fn strip_chapter_content(chapter: &mut Map<String, Value>, opts: ParseOptions) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let stripped = strip_frontmatter_with(content, opts);
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
    fn separator_and_part_title_are_left_alone() {
        let mut value = json!([
            { "PartTitle": "---\ntitle: Not a chapter\n---\n" },
            "Separator",
            {
                "Chapter": {
                    "content": "---\ntitle: After part\n---\nbody\n",
                    "sub_items": []
                }
            }
        ]);
        process_book_item(&mut value);
        assert_eq!(value[0]["PartTitle"], "---\ntitle: Not a chapter\n---\n");
        assert_eq!(value[1], "Separator");
        assert_eq!(value[2]["Chapter"]["content"], "body\n");
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
