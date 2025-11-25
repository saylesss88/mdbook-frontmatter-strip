use serde_json::{Map, Value};
use serde_yaml::Value as YamlValue;

/// Strip YAML frontmatter from a Markdown string.
/// Returns `(stripped_content, frontmatter_text)`.
pub fn strip_frontmatter(content: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (content.to_string(), None);
    }

    // Find first non-empty line; must be `---`
    let start_idx = lines.iter().position(|line| !line.trim().is_empty());
    let start_idx = match start_idx {
        Some(i) if lines[i].trim() == "---" => i,
        _ => return (content.to_string(), None),
    };

    // Find closing `---`
    let end_idx = lines
        .iter()
        .skip(start_idx + 1)
        .position(|line| line.trim() == "---")
        .map(|rel| start_idx + 1 + rel)
        .unwrap_or(lines.len());

    // Collect frontmatter
    let front = lines[start_idx..=end_idx].join("\n");

    // Collect remaining content
    let body = if end_idx + 1 < lines.len() {
        lines[end_idx + 1..].join("\n")
    } else {
        "".to_string()
    };

    (body, Some(front))
}

/// Remove keys from chapter that exist in YAML frontmatter.
fn remove_frontmatter_metadata(chapter: &mut Map<String, Value>, frontmatter: &str) {
    if let Ok(_yaml) = serde_yaml::from_str::<YamlValue>(frontmatter) {
        if let Ok(YamlValue::Mapping(map)) = serde_yaml::from_str::<YamlValue>(frontmatter) {
            for (key, _) in map {
                if let YamlValue::String(k) = key {
                    chapter.remove(&k);
                }
            }
        }
    }
}

/// Process a single Chapter object: remove frontmatter from content and metadata.
fn process_chapter(chapter: &mut Map<String, Value>) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let (stripped, frontmatter) = strip_frontmatter(content);

        // Replace content
        *content = stripped.trim_matches('\n').to_string() + "\n";

        // Remove metadata keys from YAML
        if let Some(front) = frontmatter {
            remove_frontmatter_metadata(chapter, &front);

            // Optional: debug
            eprintln!(
                "mdbook-frontmatter-strip: chapter {:?}, frontmatter removed",
                chapter
                    .get("name")
                    .unwrap_or(&Value::String("unknown".to_string()))
            );
        }
    }

    // Recurse into sub_items if present
    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for item in sub_items.iter_mut() {
            process_book_item(item);
        }
    }
}

/// Recursively process mdBook book items (sections/items/sub_items)
pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Process Chapter objects
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                process_chapter(chapter);
            }

            // Recurse into child arrays
            for key in &["sections", "items", "sub_items"] {
                if let Some(Value::Array(children)) = map.get_mut(*key) {
                    for child in children.iter_mut() {
                        process_book_item(child);
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                process_book_item(item);
            }
        }
        _ => {}
    }
}
