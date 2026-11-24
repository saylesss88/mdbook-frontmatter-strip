# mdbook-frontmatter-strip

[![Crates.io](https://img.shields.io/crates/v/mdbook-frontmatter-strip.svg)](https://crates.io/crates/mdbook-frontmatter-strip)
[![Documentation](https://docs.rs/mdbook-frontmatter-strip/badge.svg)](https://docs.rs/mdbook-frontmatter-strip)
![coverage](./coverage-report/badges/plastic.svg)

An [mdBook](https://github.com/rust-lang/mdBook) preprocessor that strips YAML
frontmatter from chapters before they are rendered, so metadata like `title`,
`date`, or `tags` doesn't show up in your book.

- Works with mdBook 0.4.x and 0.5.x (tested with 0.4.0 and 0.5.4)
- Runs for every renderer: HTML, EPUB, PDF, and others
- One dependency (`serde_json`)

Issues and PRs are welcome. If you need another frontmatter style, such as
TOML (`+++`), open an issue.

## Installation

```bash
cargo install mdbook-frontmatter-strip
```

Or install a prebuilt binary with
[`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall mdbook-frontmatter-strip
```

The binary must be on your `PATH` so mdBook can find it. Check with:

```bash
mdbook-frontmatter-strip --version
```

## Usage

Add the preprocessor to your `book.toml`:

```toml
[preprocessor.frontmatter-strip]
```

Then build as usual:

```bash
mdbook build
```

It runs for every renderer by default. To limit it to specific ones, list them:

```toml
[preprocessor.frontmatter-strip]
renderers = ["html"]
```

## Configuration

| Key        | Type | Default | Description                                                              |
| ---------- | ---- | ------- | ------------------------------------------------------------------------ |
| `unfenced` | bool | `false` | Also strip frontmatter without `---` fences (see [below](#unfenced-frontmatter-opt-in)) |

```toml
[preprocessor.frontmatter-strip]
unfenced = true
```

Unfenced detection is off by default because it can't tell metadata from
ordinary Markdown: a chapter that opens with `Note: …` and `Warning: …` lines
would lose them. Only enable it if your chapters actually use unfenced
frontmatter.

A value of the wrong type (e.g. `unfenced = "true"`) fails the build instead of
being silently ignored.

## Behavior

Frontmatter at the very top of a chapter is removed. Everything else is left as
is.

```markdown
---
title: My Chapter
tags: [rust, mdbook]
---

# My Chapter      ← the chapter now starts here
```

A chapter is only changed if it **opens with `---`, then a `key: value` line,
then a closing `---` or `...`**. These are left alone:

- A chapter that starts with a horizontal rule (`---` with no `key: value` after it)
- An opening `---` with no closing fence
- `---` lines later in the chapter
- Chapters without frontmatter (passed through byte-for-byte)

### Details

- Leading blank lines and a UTF-8 BOM before the opening `---` are fine; an
  indented `---` is not a fence.
- Fence lines may have trailing whitespace, but `----` and `--- text` are not
  fences.
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
content**. Indented lines, and `- item` lines under an empty `key:`, count as
part of the block. Chapters that start with `---` always use the fenced rules.

## Deploying

Hosted build environments (Cloudflare Pages, Netlify, etc.) won't have this
preprocessor installed, and compiling it on every deploy is slow. Download the
prebuilt Linux binary in the same build command where you install mdBook:

```bash
curl -sSL https://github.com/saylesss88/mdbook-frontmatter-strip/releases/download/v3.0.0/mdbook-frontmatter-strip-x86_64-unknown-linux-musl.tar.gz | tar xz
export PATH="$PWD:$PATH"
mdbook build
```

Pinning a version keeps deploys reproducible. To always get the newest release,
replace `download/v3.0.0` with `latest/download`.

If the binary is missing, what happens depends on your mdBook version:

- **mdBook 0.5:** the build fails. To build anyway with only a warning, add
  `optional = true` to `[preprocessor.frontmatter-strip]`.
- **mdBook 0.4:** the build *succeeds* with only a warning, and nothing is
  stripped. (If you've set `renderers`, it fails instead.)

Either way, a build that skips the preprocessor publishes your frontmatter, so
check your build log if it shows up on your site.

## Library usage

The parser is also available as a library for any Markdown string.

```toml
[dependencies]
mdbook-frontmatter-strip = "3"
```

Strip frontmatter and keep the body:

```rust
use mdbook_frontmatter_strip::strip_frontmatter;

let body = strip_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
assert_eq!(body, "Body text.\n");
```

Get the frontmatter and body separately:

```rust
use mdbook_frontmatter_strip::parse_frontmatter;

let fm = parse_frontmatter("---\ntitle: Hello\n---\n\nBody text.\n");
assert_eq!(fm.yaml, Some("title: Hello"));
assert_eq!(fm.body, "Body text.\n");
```

`yaml` is the raw YAML with the fences removed, ready for the YAML parser of
your choice. Both fields borrow from the input, so nothing is allocated.

Enable unfenced detection with `ParseOptions`:

```rust
use mdbook_frontmatter_strip::{ParseOptions, parse_frontmatter_with};

let opts = ParseOptions::new().unfenced(true);
let fm = parse_frontmatter_with("title: Hello\nauthor: Jr\n\nBody text.\n", opts);
assert_eq!(fm.yaml, Some("title: Hello\nauthor: Jr"));
assert_eq!(fm.body, "Body text.\n");
```

`strip_frontmatter_with` is the matching shortcut that returns only the body.
To process mdBook's book JSON yourself, use `process_book_item` or
`process_book_item_with`.

The [`examples/`](examples/) directory has runnable demos, including reading
from piped stdin.

## Upgrading

### To 3.0

**Preprocessor users:**

- The preprocessor now runs for every renderer, not just HTML. If your
  `book.toml` already has `renderers = ["html"]`, nothing changes; remove that
  line to strip frontmatter from EPUB, PDF, etc. too.
- mdBook 0.4 is now supported.

**Library users:**

- The `error` module is removed. No library function can fail, so none return
  a `Result`.
- The `frontmatter` module is private. Import everything from the crate root
  (`mdbook_frontmatter_strip::parse_frontmatter`).
- `is_yaml_kv` is no longer public.
- `Frontmatter` is `#[non_exhaustive]`: you can still read its fields, but you
  can't construct it with a struct literal or destructure it without `..`.

### To 2.0

Unfenced frontmatter is no longer stripped by default. If your chapters rely on
it, add `unfenced = true` to `[preprocessor.frontmatter-strip]`, or use
`ParseOptions::new().unfenced(true)` in library code. Fenced frontmatter needs
no changes.

## License

[Apache License 2.0](LICENSE)
