# mdbook-frontmatter-strip

[![Crates.io](https://img.shields.io/crates/v/mdbook-frontmatter-strip.svg)](https://crates.io/crates/mdbook-frontmatter-strip)
[![Documentation](https://docs.rs/mdbook-frontmatter-strip/badge.svg)](https://docs.rs/mdbook-frontmatter-strip)
<!-- ![coverage](https://img.shields.io/badge/coverage-91.95%25-brightgreen) -->

![coverage](./coverage-report/badges/plastic.svg)

An `mdBook` preprocessor that strips YAML frontmatter from chapters before they
are rendered, so metadata like `title`, `date`, or `tags` does not appear in the
generated HTML.

This crate is actively developed and maintained, issues and PRs are welcome. If
anyone wants a different style frontmatter, e.g. `+++`, etc. submit an issue and
I'll add it.

It's intentionally lightweight: just a single dependency (`serde_json`), so it
adds minimal weight to your build.

---

## Installation

```bash
cargo install mdbook-frontmatter-strip
```

Check your version:

```bash
mdbook-frontmatter-strip --version
```

`mdbook-frontmatter-strip` must be on your `PATH` so that `mdBook` can discover
and run it.

## Usage

Add the preprocessor to your `book.toml`:

```toml
[preprocessor.frontmatter-strip]
renderers = ["html"]
```

Then build as usual:

```bash
mdbook build
```

## Configuration

| Key | Type | Default | Description |

| --------------- | --------------- | --------------- | --------------- | |
`unfenced` | bool | `false` | Also strip unfenced frontmatter: two or more
`key: value` lines at the top of a chapter. |

```toml
[preprocessor.frontmatter-strip]
renderers = ["html"]
unfenced = true
```

Unfenced detection is off by default because it can't tell metadata from
ordinary Markdown. A chapter that opens with two lines like `Note: …` and
`Warning: …` will have them removed. Only enable it if your chapters use
unfenced frontmatter.

A non-boolean value (e.g. `unfenced = "true"`) fails the build with an error
rather than being silently ignored.

---

## Library Usage

This crate can also be used as a library to parse or strip frontmatter from
Markdown strings.

Add it to your `Cargo.toml`:

```toml
[dependencies]
mdbook-frontmatter-strip = "2"
```

Strip frontmatter and get the body:

```rust
use mdbook_frontmatter_strip::strip_frontmatter;

let body = strip_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
assert_eq!(body, "Body text.\n");
```

Access both the frontmatter and body separately:

```rust
use mdbook_frontmatter_strip::parse_frontmatter;

let fm = parse_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
assert_eq!(fm.yaml, Some("title: Hello"));
assert_eq!(fm.body, "Body text.\n");
```

The `yaml` field is a raw YAML string with fences removed, ready to pass to the
YAML parser of your choice. Both fields borrow from the input; nothing is
allocated.

Opt-in to unfenced detection with `ParseOptions`:

```rust
use mdbook_frontmatter_strip::{ParseOptions, parse_frontmatter_with};

let opts = ParseOptions::new().unfenced(true);
let fm = parse_frontmatter_with("title: Hello\nauthor: Jr\n\nBody text.\n", opts);
assert_eq!(fm.yaml, Some("title: Hello\nauthor: Jr"));
assert_eq!(fm.body, "Body text.\n");
```

`strip_frontmatter_with` is the matching shortcut that returns only the body. To
process a whole book's JSON yourself, use `process_book_item` or
`process_book_item_with`.

See the [`examples/`](examples/) directory for runnable demos, including piped
stdin input and the full `Frontmatter` API.

## Behavior

- The opening `---` must be the first non-blank line, not indented. Leading
  blank lines and a UTF-8 BOM are allowed.

- The closing fence is a line containing only `---` or `...`. Trailing
  whitespace is fine; `----` or `---` text is not a fence.

- The block must be empty or start with a key: value line (comments allowed
  before it). A chapter that merely opens with a horizontal rule (`---`) is left
  alone.

- Later `---` lines in the body are horizontal rules and are never touched.

- An empty block (--- followed by ---) is stripped, but yaml is None.

- An opening fence with no closing fence is not frontmatter; the chapter is left
  unchanged.

**Unfenced frontmatter** (opt-in, see [Configuration](#configuration)):

- Requires at least two consecutive `key: value` lines at the top.

- Never applies to a chapter whose first line is `---`.

**Both modes**:

- A `key: value` line needs whitespace (or end of line) after the colon, as in
  YAML. URLs like `http://example.com` and text like `a:b` are not keys.

- `\n` and `\r\n` line endings are both supported.

- Leading blank lines in the body are removed; indentation is kept, so an
  indented code block right after the frontmatter survives.

- Chapters without frontmatter are passed through byte-for-byte.

> Behavior is covered by both unit tests (fenced/unfenced detection, edge cases
> like top-level URLs) and integration tests (nested chapters, parts, and the
> full preprocessor pipeline), so behavior stays stable across mdBook updates
> and future changes to this crate.

---

## License

[Apache License 2.0](https://github.com/saylesss88/mdbook-frontmatter-strip/blob/main/LICENSE)
