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

> Supports mdBook 0.4.x and 0.5.x (tested with 0.4.0 and 0.5.4).

---

## Installation

```bash
cargo install mdbook-frontmatter-strip
```

Or install a prebuilt binary with `cargo-binstall`:

```bash
cargo binstall mdbook-frontmatter-strip
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
```

> Runs for every renderer by default (HTML, EPUB, PDF, etc.). To limit it, list
> the renderers you want:
> ```toml
> [preprocessor.frontmatter-strip]
> renderers = ["html"]
> ```

Then build as usual:

```bash
mdbook build
```

## Configuration

| Key        | Type | Default | Description                                                                             |
| ---------- | ---- | ------- | --------------------------------------------------------------------------------------- |
| `unfenced` | bool | `false` | Also strip unfenced frontmatter: two or more `key: value` lines at the top of a chapter |

```toml
[preprocessor.frontmatter-strip]
unfenced = true
```

Unfenced detection is off by default because it can't tell metadata from
ordinary Markdown. A chapter that opens with two lines like `Note: …` and
`Warning: …` will have them removed. Only enable it if your chapters use
unfenced frontmatter.

A non-boolean value (e.g. `unfenced = "true"`) fails the build with an error
rather than being silently ignored.

---

## Deploying

Hosted build environments (Cloudflare Pages, Netlify, etc.) don't have this
preprocessor installed, and compiling it on every deploy is slow. Download the
prebuilt Linux binary from the GitHub release instead, in the same build command
where you install `mdbook`:

```bash
curl -sSL https://github.com/saylesss88/mdbook-frontmatter-strip/releases/download/v2.0.0/mdbook-frontmatter-strip-x86_64-unknown-linux-musl.tar.gz | tar xz
export PATH="$PWD:$PATH"
mdbook build
```

Without the binary, `mdbook` build fails with an error that the preprocessor
couldn't be run. If you only want a warning, add `optional = true` to
`[preprocessor.frontmatter-strip]`. The build then succeeds, but nothing is
stripped, so your frontmatter will show up in the published site.

## Library Usage

This crate can also be used as a library to parse or strip frontmatter from
Markdown strings.

Add it to your `Cargo.toml`:

```toml
[dependencies]
mdbook-frontmatter-strip = "4"
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

Frontmatter at the very top of a chapter is removed. Everything else is left as is.

```markdown
---
title: My Chapter
tags: [rust, mdbook]
---

# My Chapter      ← the chapter now starts here
```

A chapter is only changed if it **opens with `---`, then a `key: value` line,
then a closing `---` or `...`**. So these are left alone:

- A chapter that starts with a horizontal rule (`---` with no `key: value` after it)
- An opening `---` with no closing fence
- `---` lines later in the chapter (horizontal rules)
- Chapters with no frontmatter (passed through byte-for-byte)

### Details

- Leading blank lines and a UTF-8 BOM before the opening `---` are fine; an
  indented `---` is not a fence.
- Fence lines may have trailing whitespace, but `----` or `--- text` are not fences.
- Comments may come before the first key. An empty block (`---` then `---`) is
  also stripped.
- A key needs a space or line end after the colon, as in YAML, so
  `http://example.com` and `a:b` don't count.
- Both `\n` and `\r\n` line endings work.
- Blank lines between the frontmatter and the body are removed. Indentation is
  kept, so an indented code block right after the frontmatter survives.

### Unfenced frontmatter (opt-in)

With [`unfenced = true`](#configuration), two or more `key: value` lines at the
top are stripped even without `---` fences:

```markdown
title: My Chapter
tags:
  - rust

# My Chapter      ← the chapter now starts here
```

The block ends at the first blank line, so **leave a blank line before your
content**. Indented lines and `- item` lines under an empty `key:` count as part
of the block. Chapters that start with `---` always use the fenced rules.

---

## Upgrading from 1.x

Unfenced frontmatter is no longer stripped by default. If your chapters rely on
it, add `unfenced = true` to `[preprocessor.frontmatter-strip]`, or use
`ParseOptions::new().unfenced(true)` in library code. Fenced frontmatter needs no
changes.

## License

[Apache License 2.0](https://github.com/saylesss88/mdbook-frontmatter-strip/blob/main/LICENSE)
