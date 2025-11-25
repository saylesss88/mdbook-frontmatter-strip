use serde_json::{Map, Value};

/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter must be at the very top of the file, delimited by `---` fences.
pub fn strip_frontmatter(content: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (content.to_string(), None);
    }

    // Find first non-empty line; require it to be `---`
    let mut first_nonempty = None;
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            first_nonempty = Some(i);
            break;
        }
    }
    let start_idx = match first_nonempty {
        Some(i) if lines[i].trim() == "---" => i,
        _ => return (content.to_string(), None), // no top-of-file frontmatter
    };

    // Find matching closing `---` after the start
    let end_idx = lines
        .iter()
        .skip(start_idx + 1)
        .position(|line| line.trim() == "---")
        .map(|rel| start_idx + 1 + rel)
        .unwrap_or(lines.len());

    let mut front = String::new();
    let mut body = String::new();

    // Collect frontmatter (between the fences, including them)
    for line in lines.iter().skip(start_idx).take(end_idx - start_idx + 1) {
        front.push_str(line);
        front.push('\n');
    }
    if !front.is_empty() {
        front = front.trim_end_matches('\n').to_string();
    }

    // Collect the rest as body
    for line in lines.iter().skip(end_idx + 1) {
        body.push_str(line);
        body.push('\n');
    }
    let body = body.trim().to_string();

    (
        body,
        if end_idx > start_idx {
            Some(front)
        } else {
            None
        },
    )
}

/// Process the inner `Chapter` object: strip frontmatter from its `content`.
fn process_chapter(chapter: &mut Map<String, Value>) {
    // Clone name so we don't hold an immutable borrow across get_mut
    let name = chapter
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let (stripped, frontmatter) = strip_frontmatter(content);

        // Debug log to stderr so we don't corrupt JSON
        eprintln!(
            "mdbook-frontmatter-strip: chapter={:?}, frontmatter_found={}",
            name,
            frontmatter.is_some()
        );

        *content = stripped.trim_matches('\n').to_string() + "\n";
    }

    // Recurse into nested sub_items if present
    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for item in sub_items.iter_mut() {
            process_book_item(item);
        }
    }
}

/// Recursively process mdBook "sections" / nested items.
///
/// For mdBook 0.5.x, `book["sections"]` is an array of objects like:
/// `{ "Chapter": { ... } }`, `{ "Separator": { ... } }`, etc. [web:59]
pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // If this object wraps a Chapter, process it
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                process_chapter(chapter);
            }

            // Also recurse into any nested arrays (sections, items, sub_items)
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
