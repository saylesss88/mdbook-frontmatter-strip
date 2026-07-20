//! Strip YAML frontmatter from a Markdown string.
//!
//! Frontmatter must be at the very top of the file, delimited by `---` fences,
//! or (failing that) detected heuristically as a block of unfenced `key: value`
//! lines.

fn is_yaml_kv(line: &str) -> bool {
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
}

fn normalize_body(lines: &[&str], has_trailing_nl: bool) -> String {
    let mut body = lines.join("\n");
    // Trim leading blank lines
    while body.starts_with('\n') {
        body.remove(0);
    }
    if has_trailing_nl && !body.ends_with('\n') {
        body.push('\n');
    }
    body
}

/// Returns the index into `lines` where the body starts if fenced frontmatter
/// (`---` ... `---`) is detected, starting from `start`.
fn fenced_body_start(lines: &[&str], start: usize) -> Option<usize> {
    if lines.get(start)?.trim() != "---" {
        return None;
    }
    let end = lines
        .iter()
        .skip(start + 1)
        .position(|l| l.trim() == "---")
        .map(|rel| start + 1 + rel);
    Some(end.map_or(start + 1, |e| e + 1))
}

/// Returns the index into `lines` where the body starts if unfenced YAML-like
/// lines (≥2 consecutive `key: value` lines) are detected, starting from `start`.
fn unfenced_body_start(lines: &[&str], start: usize) -> Option<usize> {
    let count = lines[start..].iter().take_while(|l| is_yaml_kv(l)).count();
    if count < 2 {
        return None;
    }
    let mut i = start + count;
    // Skip one optional blank separator line
    if lines.get(i).is_some_and(|l| l.trim().is_empty()) {
        i += 1;
    }
    Some(i)
}

/// Strip YAML frontmatter from a Markdown string.
pub fn strip_frontmatter(content: &str) -> String {
    let has_trailing_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();

    // Skip leading blank lines
    let start = lines.iter().position(|l| !l.trim().is_empty());
    let Some(start) = start else {
        return content.to_string();
    };

    let body_start =
        fenced_body_start(&lines, start).or_else(|| unfenced_body_start(&lines, start));

    body_start.map_or_else(
        || content.to_string(),
        |i| normalize_body(&lines[i..], has_trailing_nl),
    )
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

    #[test]
    fn unclosed_fence_only_strips_opening_marker() {
        // With no closing `---`, only the opening fence line is removed.
        // everything after it (including what looks like YAML) becomes body.
        let input = "---\ntitle: Hi\nbody without closing fence\n";
        assert_eq!(
            strip_frontmatter(input),
            "title: Hi\nbody without closing fence\n"
        );
    }

    #[test]
    fn preserves_missing_trailing_newline() {
        let input = "---\ntitle: Hi\n---\nbody text";
        assert_eq!(strip_frontmatter(input), "body text");
    }
}
