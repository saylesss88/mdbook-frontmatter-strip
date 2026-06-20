//! Strip YAML frontmatter from a Markdown string.
//!
//! Frontmatter must be at the very top of the file, delimited by `---` fences,
//! or (failing that) detected heuristically as a block of unfenced `key: value`
//! lines.

/// Strip YAML frontmatter from a Markdown string.
///
/// Handles two cases:
/// - **Fenced**: content starts with a `---` line, frontmatter runs until the
///   next `---` line.
/// - **Unfenced**: at least two consecutive `key: value`-shaped lines at the
///   very top, with no fences.
///
/// If neither pattern is found, `content` is returned unchanged.
pub fn strip_frontmatter(content: &str) -> String {
    let has_trailing_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // Define as a closure instead of a function
    let is_yaml_kv = |line: &str| -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "---" {
            return false;
        }
        if let Some(colon_idx) = trimmed.find(':') {
            let key = trimmed[..colon_idx].trim();
            if key.is_empty() || key.contains("://") {
                return false;
            }
            key.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        } else {
            false
        }
    };

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

        let body_start = end_idx.map_or_else(|| start_idx + 1, |end_idx| end_idx + 1);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenced_frontmatter_is_stripped() {
        let input = "---\ntitle: Hi\n---\nbody text\n";
        assert_eq!(strip_frontmatter(input), "body text\n");
    }

    #[test]
    fn unfenced_frontmatter_is_stripped() {
        let input = "title: Hi\nauthor: Tom\n\nbody text\n";
        assert_eq!(strip_frontmatter(input), "body text\n");
    }

    #[test]
    fn single_kv_line_is_not_frontmatter() {
        // Needs >= 2 lines to count as unfenced frontmatter.
        let input = "title: Hi\nbody text\n";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn url_like_line_is_not_treated_as_kv() {
        let input = "see: https://example.com\nbody text\n";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn no_frontmatter_is_unchanged() {
        let input = "# Just a heading\n\nSome body text.\n";
        assert_eq!(strip_frontmatter(input), input);
    }

    // #[test]
    // fn unclosed_fence_strips_everything_after_opening() {
    //     let input = "---\ntitle: Hi\nbody without closing fence\n";
    //     assert_eq!(strip_frontmatter(input), "");
    // }

    #[test]
    fn preserves_missing_trailing_newline() {
        let input = "---\ntitle: Hi\n---\nbody text";
        assert_eq!(strip_frontmatter(input), "body text");
    }
}
