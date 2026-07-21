# mdbook-frontmatter-strip

[![Crates.io](https://img.shields.io/crates/v/mdbook-frontmatter-strip.svg)](https://crates.io/crates/mdbook-frontmatter-strip)
[![Documentation](https://docs.rs/mdbook-frontmatter-strip/badge.svg)](https://docs.rs/mdbook-frontmatter-strip)
![coverage](https://img.shields.io/badge/coverage-91.95%25-brightgreen)

An mdBook preprocessor that strips YAML frontmatter from chapters before they
are rendered, so metadata like `title`, `date`, or `tags` does not appear in the
generated HTML.

This crate is actively developed and maintained, issues and PRs are welcome.

It's intentionally lightweight: just two dependencies (`serde_json` and
`anyhow`), so it adds minimal weight to your build.

---

## Installation

```bash
cargo install mdbook-frontmatter-strip
```

Check your version:

```bash
mdbook-frontmatter-strip --version
```

`mdbook-frontmatter-strip` must be on your `PATH` so that `mdbook` can discover
and run it.

Tested with:

- mdbook v0.5.4
- Rust editions 2020 & 2024

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

## Behavior

- Supports fenced `---` YAML frontmatter at the top.
- Supports unfenced YAML only when there are at least 2 consecutive `key: value`
  lines.
- URLs like `http://example.com` at the top are not treated as frontmatter.

> Behavior is covered by both unit tests (fenced/unfenced detection, edge cases
> like top-level URLs) and integration tests (nested chapters, parts, and the
> full preprocessor pipeline), so behavior stays stable across mdBook updates
> and future changes to this crate.

---

## License

[Apache License 2.0](https://github.com/saylesss88/mdbook-frontmatter-strip/blob/main/LICENSE)
