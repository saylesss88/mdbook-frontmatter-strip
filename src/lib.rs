use serde_json::{Map, Value};

/// Strip YAML frontmatter from a Markdown string.
/// Frontmatter is assumed to be the first `---` block in the file.
pub fn strip_frontmatter(content: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (content.to_string(), None);
    }

    // Find first line that is exactly a '---' fence (ignoring surrounding whitespace)
    let start_idx = match lines.iter().position(|line| line.trim() == "---") {
        Some(i) => i,
        None => return (content.to_string(), None), // no frontmatter
    };

    // Find matching closing '---' after the start fence
    let end_idx = lines
        .iter()
        .skip(start_idx + 1)
        .position(|line| line.trim() == "---")
        .map(|rel| start_idx + 1 + rel)
        .unwrap_or(lines.len());

    let mut front = String::new();
    let mut body = String::new();

    // Collect frontmatter (between the fences, including them)
    for line in lines.iter().skip(start_idx).take(end_idx - start_idx) {
        front.push_str(line);
        front.push('\n');
    }
    if !front.is_empty() {
        front = front.trim_end_matches('\n').to_string();
    }

    // Collect the rest as body
    for line in lines.iter().skip(end_idx) {
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

// Process a single Chapter object: strip frontmatter from its `content`.
fn process_chapter(chapter: &mut Map<String, Value>) {
    // 1. Take an owned copy of the name so we don't hold a borrow into `chapter`
    let name = chapter
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string(); // <-- clone into owned String; borrow ends here

    // 2. Now it's safe to mutably borrow `chapter` for `content`
    if let Some(Value::String(content)) = chapter.get_mut("content") {
        let (stripped, frontmatter) = strip_frontmatter(content);

        println!(
            "Processing chapter: {}, frontmatter found: {}",
            name,
            frontmatter.is_some()
        );

        *content = stripped.trim_matches('\n').to_string() + "\n";
    }

    // 3. Recurse into nested items if needed
    if let Some(Value::Array(sub_items)) = chapter.get_mut("sub_items") {
        for item in sub_items.iter_mut() {
            process_book_item(item);
        }
    }
}

/// Recursively process any mdBook "section" value.
///
/// mdBook 0.5.x represents sections as an array under `book["sections"]`,
/// where each element is an object like `{ "Chapter": { ... } }`,
/// `{ "Separator": { ... } }`, etc. [web:59][web:44]
pub fn process_book_item(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // If this object wraps a Chapter, drill into it
            if let Some(Value::Object(chapter)) = map.get_mut("Chapter") {
                process_chapter(chapter);
            }

            // Recurse into nested arrays if present (sections, items, sub_items, etc.)
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
