//! Strip YAML frontmatter from a Markdown string.
//!
//! Frontmatter must be at the very top of the file, delimited by `---` fences,
//! or (failing that) detected heuristically as a block of unfenced `key: value`
//! lines.

/// The result of parsing frontmatter from a Markdown string.
#[derive(Debug)]
pub struct Frontmatter {
    /// The raw YAML string between the `---` fences, or the heuristically
    /// detected `key: value` block. `None` if no frontmatter was found.
    pub yaml: Option<String>,
    /// Everything after the frontmatter, with leading blank lines stripped.
    pub body: String,
}

impl Frontmatter {
    /// Returns the YAML frontmatter if present, otherwise returns the full body.
    #[must_use]
    pub fn yaml_or_body(&self) -> &str {
        self.yaml.as_deref().unwrap_or(&self.body)
    }

    /// Returns `true` if frontmatter was detected.
    #[must_use]
    pub const fn has_frontmatter(&self) -> bool {
        self.yaml.is_some()
    }
}
/// Parse and return both the frontmatter and body separately.
///
/// # Examples
/// ```
///
/// # use mdbook_frontmatter_strip::parse_frontmatter;
/// let fm = parse_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
/// assert_eq!(fm.yaml.as_deref(), Some("title: Hello"));
/// assert_eq!(fm.body, "Body text.\n");
/// ```
#[must_use]
pub fn parse_frontmatter(content: &str) -> Frontmatter {
    let has_trailing_nl = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();

    let Some(start) = lines.iter().position(|l| !l.trim().is_empty()) else {
        return Frontmatter {
            yaml: None,
            body: content.to_string(),
        };
    };

    // Check if it's fenced frontmatter
    if lines.get(start).is_some_and(|l| l.trim() == "---") {
        if let Some(body_start) = fenced_body_start(&lines, start) {
            let end = body_start - 1;
            let yaml = lines[start + 1..end].join("\n");
            let yaml = yaml.trim().to_string();

            return Frontmatter {
                yaml: if yaml.is_empty() { None } else { Some(yaml) },
                body: normalize_body(&lines[body_start..], has_trailing_nl),
            };
        }

    // Don't fall through to unfenced detection if we saw an opening fence,
    // an unclosed fence should not be silently treated as unfenced YAML.
    } else {
        // Compute count once; reuse it for both the guard and the yaml slice.
        let count = lines[start..].iter().take_while(|l| is_yaml_kv(l)).count();
        if count >= 2 {
            let body_start = unfenced_body_start(&lines, start, count);
            let yaml = lines[start..start + count].join("\n");
            return Frontmatter {
                yaml: Some(yaml),
                body: normalize_body(&lines[body_start..], has_trailing_nl),
            };
        }
    }
    Frontmatter {
        yaml: None,
        body: content.to_string(),
    }
}

/// Is this a YAML key-value pair?
#[must_use]
pub fn is_yaml_kv(line: &str) -> bool {
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
    let body = lines.join("\n");
    let body = body.trim_start_matches('\n');
    let mut body = body.to_string();
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
        .map(|rel| start + 1 + rel)?; // require closing fence
    Some(end + 1)
}

/// Returns the index into `lines` where the body starts if unfenced YAML-like
/// lines (≥2 consecutive `key: value` lines) are detected, starting from `start`.
fn unfenced_body_start(lines: &[&str], start: usize, count: usize) -> usize {
    let mut i = start + count;
    // Skip one optional blank separator line
    if lines.get(i).is_some_and(|l| l.trim().is_empty()) {
        i += 1;
    }
    i
}

/// Strip YAML frontmatter from a Markdown string.
#[must_use]
pub fn strip_frontmatter(content: &str) -> String {
    parse_frontmatter(content).body
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
    fn empty_fenced_frontmatter_yields_none_yaml() {
        // ---\n---\n should not panic and should give yaml: None
        let fm = parse_frontmatter("---\n---\nbody\n");
        assert!(fm.yaml.is_none());
        assert_eq!(fm.body, "body\n");
    }

    #[test]
    fn frontmatter_with_leading_blank_lines() {
        // Some generators emit a blank line before ---
        let input = "\n---\ntitle: Hi\n---\nbody\n";
        let fm = parse_frontmatter(input);
        assert_eq!(fm.yaml.as_deref(), Some("title: Hi"));
    }

    #[test]
    fn fenced_frontmatter_with_multiline_value() {
        // Folded/literal block scalars are common in Hugo/Jekyll
        let input = "---\ntitle: Hi\ntags:\n  - rust\n  - mdbook\n---\nbody\n";
        let fm = parse_frontmatter(input);
        assert!(fm.yaml.is_some()); // just check it doesn't panic/lose data
    }

    #[test]
    fn preserves_missing_trailing_newline() {
        let input = "---\ntitle: Hi\n---\nbody text";
        assert_eq!(strip_frontmatter(input), "body text");
    }

    #[test]
    fn fenced_yaml_is_returned_without_delimiters() {
        let fm = parse_frontmatter("---\ntitle: Hi\ndate: 2024\n---\nbody\n");
        assert_eq!(fm.yaml.as_deref(), Some("title: Hi\ndate: 2024"));
    }

    #[test]
    fn unfenced_yaml_is_returned() {
        let fm = parse_frontmatter("title: Hi\nauthor: Jr\n\nbody\n");
        assert_eq!(fm.yaml.as_deref(), Some("title: Hi\nauthor: Jr"));
    }

    #[test]
    fn no_frontmatter_yields_none_yaml() {
        let fm = parse_frontmatter("# Heading\n\nbody\n");
        assert!(fm.yaml.is_none());
    }
}
