//! Detect and strip YAML frontmatter from Markdown.
//!
//! Only *fenced* frontmatter is recognized:
//!
//! ```text
//! ---
//! title: Hello
//! ---
//! ```

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
        self.yaml.unwrap_or(self.body)
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
    let none = Frontmatter {
        yaml: None,
        body: content,
    };
    let src = content.strip_prefix('\u{feff}').unwrap_or(content);

    let mut lines = lines_with_offsets(src);
    let Some((_, first, yaml_start)) = lines.next() else {
        return none;
    };
    if first.trim_end() != "---" {
        return none;
    }

    for (start, line, end) in lines {
        let fence = line.trim_end();
        if fence == "---" || fence == "..." {
            let yaml = strip_eol(&src[yaml_start..start]);
            if !looks_like_yaml_mapping(yaml) {
                return none;
            }
            return Frontmatter {
                yaml: Some(yaml),
                body: skip_blank_lines(&src[end..]),
            };
        }
    }

    // Opening fence without a closing one: not frontmatter.
    none
}

/// Is this line a YAML `key: value` (or `key:`) mapping entry?
///
/// Unquoted keys may contain alphanumerics, `_`, `-` and `.`; quoted keys
/// (`"a b": 1`, `'x': y`) may contain anything. The colon must be followed
/// by whitespace or end of line, as YAML requires, so `http://x` and
/// `a:b` are rejected.
#[must_use]
pub fn is_yaml_kv(line: &str) -> bool {
    let line = line.trim();

    let after_key = if let Some(q @ ('"' | '\'')) = line.chars().next() {
        let inner = &line[1..];
        let Some(i) = inner.find(q) else {
            return false;
        };
        &inner[i + 1..]
    } else {
        let Some(i) = line.find(':') else {
            return false;
        };
        let key = &line[..i];
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            return false;
        }
        &line[i..]
    };

    after_key
        .strip_prefix(':')
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
}

/// Strip YAML frontmatter from a Markdown string.
#[must_use]
pub fn strip_frontmatter(content: &str) -> &str {
    parse_frontmatter(content).body
}

/// Empty, or the first non-blank, non-comment line is a mapping key.
/// A block containing only comments is rejected: `# Heading` between two
/// thematic breaks is far more likely Markdown than YAML.
fn looks_like_yaml_mapping(yaml: &str) -> bool {
    if yaml.trim().is_empty() {
        return true;
    }
    yaml.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .is_some_and(is_yaml_kv)
}

/// Yields `(line_start, line_without_eol, next_line_start)`.
fn lines_with_offsets(s: &str) -> impl Iterator<Item = (usize, &str, usize)> {
    let mut pos = 0;
    s.split_inclusive('\n').map(move |raw| {
        let start = pos;
        pos += raw.len();
        (start, strip_eol(raw), pos)
    })
}

fn strip_eol(s: &str) -> &str {
    let s = s.strip_suffix('\n').unwrap_or(s);
    s.strip_suffix('\r').unwrap_or(s)
}

/// Remove whole blank lines from the start, keeping indentation of the first
/// real line (so a leading indented code block survives).
fn skip_blank_lines(mut s: &str) -> &str {
    while let Some(i) = s.find('\n') {
        if s[..i].trim().is_empty() {
            s = &s[i + 1..];
        } else {
            return s;
        }
    }
    if s.trim().is_empty() { "" } else { s }
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
        assert_eq!(fm.yaml, Some("title: Hi"));
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
        assert_eq!(fm.yaml, Some("title: Hi\ndate: 2024"));
    }

    #[test]
    fn unfenced_yaml_is_returned() {
        let fm = parse_frontmatter("title: Hi\nauthor: Jr\n\nbody\n");
        assert_eq!(fm.yaml, Some("title: Hi\nauthor: Jr"));
    }

    #[test]
    fn no_frontmatter_yields_none_yaml() {
        let fm = parse_frontmatter("# Heading\n\nbody\n");
        assert!(fm.yaml.is_none());
    }
}
