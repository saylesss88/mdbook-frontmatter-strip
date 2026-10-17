//! Detect and strip YAML frontmatter from Markdown.
//!
//! *Fenced* frontmatter is always recognized:
//!
//! ```text
//! ---
//! title: Hello
//! ---
//! ```
//!
//! Rules:
//! - The opening `---` must be the first non-blank line, unindented. An
//!   optional UTF-8 BOM and blank lines may precede it. Trailing whitespace
//!   on a fence line is tolerated.
//! - The closing fence is a line that is exactly `---` or `...`.
//! - The block must be empty or start with a `key: value` line, so a document
//!   that merely opens with a thematic break (`---`) isn't mistaken for
//!   frontmatter.
//! - `\n` and `\r\n` line endings are both supported.
//! - If no frontmatter is found, the input is returned **unchanged**.
//!
//! *Unfenced* frontmatter (two or more `key: value` lines at the top of
//! the file, no `---`) is opt-in via [`ParseOptions::unfenced`], because
//! it can strip ordinary markdown such as `Note:: ...`/ `Warning: ...` lines.

/// Options for [`parse_frontmatter_with`].
///
/// ```
/// # use mdbook_frontmatter_strip::{parse_frontmatter_with, ParseOptions};
/// let opts = ParseOptions::new().unfenced(true);
/// let fm = parse_frontmatter_with("title: Hi\nauthor: Me\n\nBody\n", opts);
/// assert_eq!(fm.yaml, Some("title: Hi\nauthor: Me"));
/// assert_eq!(fm.body, "Body\n");
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseOptions {
    /// Detect unfenced frontmatter: two or more consecutive `key: value`
    /// lines at the start of the document. Off by default.
    pub unfenced: bool,
}

impl ParseOptions {
    /// Default options: fenced frontmatter only.
    #[must_use]
    pub const fn new() -> Self {
        Self { unfenced: false }
    }

    /// Enable or disable unfenced frontmatter detection.
    #[must_use]
    pub const fn unfenced(mut self, yes: bool) -> Self {
        self.unfenced = yes;
        self
    }
}

/// The result of parsing frontmatter from a Markdown string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Parse fenced frontmatter and return it and the body separately.
///
/// Equivalent to [`parse_frontmatter_with`] with default [`ParseOptions`].
///
/// # Examples
/// ```
/// # use mdbook_frontmatter_strip::parse_frontmatter;
/// let fm = parse_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
/// assert_eq!(fm.yaml, Some("title: Hello"));
/// assert_eq!(fm.body, "Body text.\n");
///
/// // A leading thematic break is not frontmatter.
/// let md = "---\n\nJust a paragraph.\n\n---\n";
/// assert_eq!(parse_frontmatter(md).body, md);
/// ```
#[must_use]
pub fn parse_frontmatter(content: &str) -> Frontmatter<'_> {
    parse_frontmatter_with(content, ParseOptions::new())
}

/// Parse frontmatter using the given options.
///
/// Fenced fm always takes priority. A document with a first line of
/// `---` never falls through to unfenced detection, even if the fence
/// is unclosed
#[must_use]
pub fn parse_frontmatter_with(content: &str, opts: ParseOptions) -> Frontmatter<'_> {
    let none = Frontmatter {
        yaml: None,
        body: content,
    };
    let src = content.strip_prefix('\u{feff}').unwrap_or(content);
    let src = skip_blank_lines(src);

    let Some((_, first, _)) = lines_with_offsets(src).next() else {
        return none;
    };
    if first.trim_end() == "---" {
        return parse_fenced(src).unwrap_or(none);
    }
    if opts.unfenced
        && let Some(fm) = parse_unfenced(src)
    {
        return fm;
    }
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

/// Strip YAML frontmatter from a Markdown string. Returns the input unchanged if there is none.
#[must_use]
pub fn strip_frontmatter(content: &str) -> &str {
    parse_frontmatter(content).body
}

/// Strip YAML frontmatter using given options.
#[must_use]
pub fn strip_frontmatter_with(content: &str, opts: ParseOptions) -> &str {
    parse_frontmatter_with(content, opts).body
}

/// `src` starts with an opening `---` line.
fn parse_fenced(src: &str) -> Option<Frontmatter<'_>> {
    let mut lines = lines_with_offsets(src);
    let (_, _, yaml_start) = lines.next()?;
    for (start, line, end) in lines {
        let fence = line.trim_end();
        if fence == "---" || fence == "..." {
            let yaml = strip_eol(&src[yaml_start..start]);
            if !looks_like_yaml_mapping(yaml) {
                return None;
            }
            return Some(Frontmatter {
                yaml: (!yaml.trim().is_empty()).then_some(yaml),
                body: skip_blank_lines(&src[end..]),
            });
        }
    }
    // Opening fence without a closing one: not frontmatter.
    None
}

/// Two or more consecutive `key: value` lines at the start of `src`
/// (the 1.0 heuristic).
fn parse_unfenced(src: &str) -> Option<Frontmatter<'_>> {
    let mut keys = 0;
    let mut yaml_end = 0;
    let mut body_start = 0;
    let mut open_list = false;

    for (start, line, next) in lines_with_offsets(src) {
        if line.trim().is_empty() {
            break;
        }
        let indented = line.starts_with([' ', '\t']);
        let list_item = {
            let t = line.trim_end();
            t == "-" || t.starts_with("- ")
        };

        if keys > 0 && indented {
            // Continuation of the previous key; `open_list` is unchanged.
        } else if keys > 0 && open_list && list_item {
            // `tags:` followed by `- a` at column 0 (valid YAML).
        } else if !indented && is_yaml_kv(line) {
            keys += 1;
            open_list = line.trim_end().ends_with(':');
        } else {
            break;
        }
        yaml_end = start + line.len(); // excludes `\r\n` / `\n`
        body_start = next;
    }

    (keys >= 2).then(|| Frontmatter {
        yaml: Some(&src[..yaml_end]),
        body: skip_blank_lines(&src[body_start..]),
    })
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

    const ON: ParseOptions = ParseOptions::new().unfenced(true);

    fn uf(s: &str) -> (Option<&str>, &str) {
        let f = parse_frontmatter_with(s, ON);
        (f.yaml, f.body)
    }

    fn fm(s: &str) -> (Option<&str>, &str) {
        let f = parse_frontmatter(s);
        (f.yaml, f.body)
    }

    #[test]
    fn fence_must_be_whole_line() {
        // `----` / `---title` / `--- x` are not fences.
        for s in [
            "----\na: 1\n----\nB",
            "---a: 1\n---\nB",
            "--- x\na: 1\n---\nB",
        ] {
            assert_eq!(fm(s), (None, s), "{s:?}");
        }
        // Closing `----` or `---x` inside the block isn't a close.
        let s = "---\na: 1\n----\nb: 2\n---x\n";
        assert_eq!(fm(s), (None, s));
    }

    #[test]
    fn fenced_frontmatter_is_stripped() {
        let input = "---\ntitle: Hi\n---\nbody text\n";
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
    fn no_frontmatter_yields_none_yaml() {
        let fm = parse_frontmatter("# Heading\n\nbody\n");
        assert!(fm.yaml.is_none());
    }
    #[test]
    fn indented_list_under_key() {
        let s = "title: Results Cache\n\
                 date: 2026-10-17\n\
                 author: saylesss88\n\
                 author-email: saylesss87@proton.me\n\
                 tags:\n\
                 \x20 - rust-analyzer\n\
                 \x20 - rust-meth-lib\n\
                 lang: en-US\n\
                 \n\
                 # Results Cache\n";
        let (yaml, body) = uf(s);
        assert_eq!(body, "# Results Cache\n");
        let yaml = yaml.unwrap();
        assert!(yaml.starts_with("title: Results Cache\n"), "{yaml}");
        assert!(yaml.contains("  - rust-meth-lib\nlang: en-US"), "{yaml}");
        assert!(yaml.ends_with("lang: en-US"), "{yaml}");
    }

    #[test]
    fn unindented_list_under_key() {
        assert_eq!(
            uf("title: X\ntags:\n- a\n- b\nlang: en\n\nBody\n"),
            (Some("title: X\ntags:\n- a\n- b\nlang: en"), "Body\n")
        );
    }

    #[test]
    fn nested_map_and_block_scalar() {
        assert_eq!(
            uf(
                "title: X\nauthor:\n  name: Jr\n  email: t@x\ndesc: |\n  line one\n  line two\n\nBody"
            ),
            (
                Some(
                    "title: X\nauthor:\n  name: Jr\n  email: t@x\ndesc: |\n  line one\n  line two"
                ),
                "Body"
            )
        );
    }

    #[test]
    fn markdown_list_after_kv_lines_is_body() {
        // `- item` under a key that already has a value is Markdown, not YAML.
        assert_eq!(
            uf("title: X\nlang: en\n- a markdown list\n"),
            (Some("title: X\nlang: en"), "- a markdown list\n")
        );
    }

    #[test]
    fn list_items_dont_count_as_keys() {
        // Only one real key: not frontmatter.
        let s = "tags:\n  - a\n  - b\n\nBody\n";
        assert_eq!(uf(s), (None, s));
        let s = "Note:\n- one\n- two\n\nBody\n";
        assert_eq!(uf(s), (None, s));
    }
}
