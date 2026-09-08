//! Strip YAML frontmatter from a Markdown string.
//!
//! Frontmatter must be at the very top of the file, delimited by `---` fences,
//! or (failing that) detected heuristically as a block of unfenced `key: value`
//! lines.

/// The result of parsing frontmatter from a Markdown string.
#[derive(Debug)]
pub struct Frontmatter<'a> {
    /// The raw YAML string between the `---` fences, or the heuristically
    /// detected `key: value` block. `None` if no frontmatter was found.
    pub yaml: Option<&'a str>,
    /// Everything after the frontmatter, with leading blank lines stripped.
    pub body: &'a str,
}

impl Frontmatter<'_> {
    /// Returns the YAML frontmatter if present, otherwise returns the full body.
    #[must_use]
    pub fn yaml_or_body(&self) -> &str {
        self.yaml.unwrap_or(&self.body)
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
pub fn parse_frontmatter(content: &str) -> Frontmatter<'_> {
    let content = content.trim_start();

    // Try fenced frontmatter first: --- ... ---
    if content.starts_with("---") {
        if let Some(rest) = content.strip_prefix("---")
            && let Some((yaml, body)) = rest.split_once("\n---")
        {
            let yaml = yaml.trim();
            let body = body.trim_start();

            return Frontmatter {
                yaml: if yaml.is_empty() { None } else { Some(yaml) },
                body,
            };
        }

        // Saw an opening fence but no closing fence, Bdo not fall through to
        // unfenced detection; return the content as-is.
        return Frontmatter {
            yaml: None,
            body: content,
        };
    }

    if let Some(yaml_end) = unfenced_yaml_end(content) {
        let yaml = &content[..yaml_end];
        let rest = content[yaml_end..].trim_start_matches('\n');
        // Skip one optional blank separator line between YAML and body.
        let body = rest.strip_prefix('\n').unwrap_or(rest);
        return Frontmatter {
            yaml: Some(yaml),
            body,
        };
    }
    Frontmatter {
        yaml: None,
        body: content,
    }
}

/// Returns the byte offset of the end of an unfenced YAML block (i.e. ≥2
/// consecutive `key: value` lines) within `content`, or `None` if no such
/// block is found.
fn unfenced_yaml_end(content: &str) -> Option<usize> {
    let mut offset = 0;
    let mut kv_count = 0;
    let mut kv_end = 0;

    for line in content.lines() {
        if is_yaml_kv(line) {
            kv_count += 1;
            kv_end = offset + line.len();
        } else {
            break;
        }
        offset += line.len() + 1; // +1 for '\n'
    }

    if kv_count >= 2 { Some(kv_end) } else { None }
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

/// Strip YAML frontmatter from a Markdown string.
#[must_use]
pub fn strip_frontmatter(content: &str) -> &str {
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
