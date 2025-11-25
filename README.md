# mdbook-frontmatter-strip

An mdBook preprocessor that strips YAML frontmatter from chapters before they
are rendered, so metadata like `title`, `date`, or `tags` does not appear in the
generated HTML.

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

## License

Apache-2.0
