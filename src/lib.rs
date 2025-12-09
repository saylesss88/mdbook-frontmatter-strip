/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter must be at the very top of the file, delimited by `---` fences.
use serde_json::{Map, Value};

fn strip_frontmatter(content: &str) -> String {
    let has_trailing_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    fn is_yaml_kv(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "---" {
            return false;
        }
        if let Some(colon_idx) = trimmed.find(':') {
            let (key, _) = trimmed.split_at(colon_idx);
            let key = key.trim();
            if key.is_empty() {
                return false;
            }
            if key.contains("://") {
                return false;
            }
            key.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        } else {
            false
        }
    }

    // Skip leading empty lines
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

        let end_idx = lines
            .iter()
            .skip(start_idx + 1)
            .position(|line| line.trim() == "---")
            .map(|rel| start_idx + 1 + rel);

        let body_start = if let Some(end_idx) = end_idx {
            end_idx + 1
        } else {
            start_idx + 1
        };

        let mut body = lines[body_start..].join("\n");
        // Remove leading blank line from body if present
        while body.starts_with('\n') {
            body.remove(0);
        }
        if has_trailing_nl && !body.ends_with('\n') {
            body.push('\n');
        }
        return body;
    }

    // Case B: unfenced YAML-like lines at the very top
    let mut front_lines = 0;
    let mut i = idx;
    while i < lines.len() && is_yaml_kv(lines[i]) {
        front_lines += 1;
        i += 1;
    }

    if front_lines >= 2 {
        if i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
        let mut body = lines[i..].join("\n");
        while body.starts_with('\n') {
            body.remove(0);
        }
        if has_trailing_nl && !body.ends_with('\n') {
            body.push('\n');
        }
        return body;
    }

    content.to_string()
}

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
