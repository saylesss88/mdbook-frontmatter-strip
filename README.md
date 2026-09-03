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

It's intentionally lightweight: just one dependency (`serde_json`), so it adds
minimal weight to your build.

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

---


## Library Usage

This crate can also be used as a library to parse or strip frontmatter from
Markdown strings.

Add it to your `Cargo.toml`:

```toml
[dependencies]
mdbook-frontmatter-strip = { version = "1.2", default-features = false }
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
println!("{}", fm.yaml.unwrap()); // title: Hello
println!("{}", fm.body);          // Body text.
```

The `yaml` field is a raw YAML string with fence delimiters stripped, ready to
pass to `yaml_serde` or any other YAML parser of your choice.

See the [`examples/`](examples/) directory for runnable demos, including piped
stdin input and the full `Frontmatter` API.

---

## Behavior

- Supports fenced `---` YAML frontmatter at the top.
- Supports unfenced YAML only when there are at least 2 consecutive `key: value`
  lines.
- URLs like `http://example.com` at the top are not treated as frontmatter.

> Behavior is covered by both unit tests (fenced/unfenced detection, edge cases
> like top-level URLs) and integration tests (nested chapters, parts, and the
> full preprocessor pipeline), so behavior stays stable across `mdBook` updates
> and future changes to this crate.

---

## License

[Apache License 2.0](https://github.com/saylesss88/mdbook-frontmatter-strip/blob/main/LICENSE)
