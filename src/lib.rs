use serde_json::{Map, Value};

/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter must be at the very top of the file, delimited by `---` fences.
pub fn strip_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // Find the first non-empty line
    let start_idx = lines
        .iter()
        .position(|line| !line.trim().is_empty())
        .filter(|&i| lines[i].trim() == "---");

    if start_idx.is_none() {
        return content.to_string();
    }
    let start_idx = start_idx.unwrap();

    // Find matching closing ---
    let end_idx = lines
        .iter()
        .skip(start_idx + 1)
        .position(|line| line.trim() == "---")
        .map(|rel| start_idx + 1 + rel);

    let body = if let Some(end_idx) = end_idx {
        lines
            .iter()
            .skip(end_idx + 1)
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        lines
            .iter()
            .skip(start_idx + 1)
            .cloned()
            .collect::<Vec<&str>>()
            .join("\n")
    };

    body.trim_start_matches('\n').to_string()
}

/// Process a chapter object, removing YAML frontmatter from its content
fn process_chapter(chapter: &mut Map<String, Value>) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let stripped = strip_frontmatter(content);
        *content = stripped.trim_matches('\n').to_string() + "\n";
    }

    // Recurse into sub_items if present
    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for item in sub_items.iter_mut() {
            process_book_item(item);
        }
    }
}

/// Recursively process all mdBook items (chapters, parts, sections)
pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Process Chapter if present
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                process_chapter(chapter);
            }
            // Also process Part (may contain sections)
            if let Some(Value::Object(part)) = map.get_mut("Part") {
                for key in &["sections", "items", "sub_items"] {
                    if let Some(Value::Array(children)) = part.get_mut(*key) {
                        for child in children.iter_mut() {
                            process_book_item(child);
                        }
                    }
                }
            }

            // Recurse into any arrays
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
