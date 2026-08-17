# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.4] - 2026-08-17

### Added

- This CHANGELOG

- `cargo-fuzz`/`arbitrary`: to fuzz with raw bytes and structured input.

```sh
cargo fuzz run strip_frontmatter
cargo fuzz run structured_strip
```

### Fixed

- `yaml_frontmatter.rs`: `normalize_body` does `body.remove(0)` in a loop which
  could lead to a panic and slow evaluation. Replaced with `trim_start_matches`.

## [1.1.3] - 2026-08-17

### Added

- new clippy lints

- Exclude section to `Cargo.toml`

- Test coverage badge in `README.md`.

### Removed

- `main.rs`: remove redundant else block in `Command::Supports`

### Fixed

- cli: USAGE section as a `const` rather than many `eprintln!`

### Changed

- refactor(frontmatter): create helper functions to slim down
  `strip_frontmatter` function.
