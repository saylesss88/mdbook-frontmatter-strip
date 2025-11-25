use serde_json::{Map, Value};

/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter must be at the very top of the file, delimited by `---` fences.
pub fn strip_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // Helper: does this line look like a YAML key: value?
    fn is_yaml_kv(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }
        // crude but effective: starts with an identifier-ish token followed by :
        if let Some(colon_idx) = trimmed.find(':') {
            let (key, _) = trimmed.split_at(colon_idx);
            // avoid matching things like "http://"
            key.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                && !key.is_empty()
        } else {
            false
        }
    }

    // 1) Skip leading empty lines
    let mut idx = 0;
    while idx < lines.len() && lines[idx].trim().is_empty() {
        idx += 1;
    }

    if idx >= lines.len() {
        return content.to_string();
    }

    // Case A: fenced frontmatter starting with ---
    if lines[idx].trim() == "---" {
        let start_idx = idx;

        // Find matching closing ---
        let end_idx = lines
            .iter()
            .skip(start_idx + 1)
            .position(|line| line.trim() == "---")
            .map(|rel| start_idx + 1 + rel);

        let body_start = if let Some(end_idx) = end_idx {
            end_idx + 1
        } else {
            // No closing fence: treat everything after the first fence as body
            start_idx + 1
        };

        let body = lines[body_start..].join("\n");
        return body.trim_start_matches('\n').to_string();
    }

    // Case B: unfenced YAML-like lines at the very top
    let mut front_lines = 0;
    let mut i = idx;
    while i < lines.len() && is_yaml_kv(lines[i]) {
        front_lines += 1;
        i += 1;
    }

    if front_lines > 0 {
        // Optionally skip a single blank line after the header block
        if i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
        let body = lines[i..].join("\n");
        return body.trim_start_matches('\n').to_string();
    }

    // Otherwise, no frontmatter detected
    content.to_string()
}
/// Process a chapter object, removing YAML frontmatter from its content
fn process_chapter(chapter: &mut Map<String, Value>) {
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let stripped = strip_frontmatter(content);
        *content = stripped.trim_matches('\n').to_string() + "\n";
    }

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
