# mdbook-frontmatter-strip

An mdBook preprocessor that strips YAML frontmatter from chapters before they
are rendered, so metadata like `title`, `date`, or `tags` does not appear in the
generated HTML.

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

- mdbook v0.5.1

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

> This crate includes integration tests that exercise fenced and unfenced
> frontmatter stripping, nested chapters, and common edge cases (like top-level
> URLs), to keep the behavior stable across mdBook updates.

---

## License

[Apache License 2.0](https://github.com/saylesss88/mdbook-frontmatter-strip/blob/main/LICENSE)
