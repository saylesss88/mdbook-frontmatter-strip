mod frontmatter;
/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter must be at the very top of the file, delimited by `---` fences.
use serde_json::{Map, Value};

use crate::frontmatter::strip_frontmatter;

fn process_chapter(chapter: &mut Map<String, Value>) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        *content = strip_frontmatter(content);
    }

    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for item in sub_items {
            process_book_item(item);
        }
    }
}

pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                process_chapter(chapter);
            }

            if let Some(Value::Object(part)) = map.get_mut("Part")
                && let Some(Value::Array(children)) = part.get_mut("sections")
            {
                for child in children {
                    process_book_item(child);
                }
            }

            for key in &["sections", "items", "sub_items"] {
                if let Some(Value::Array(children)) = map.get_mut(*key) {
                    for child in children {
                        process_book_item(child);
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
