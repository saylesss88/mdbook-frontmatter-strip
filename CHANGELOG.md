# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.2] - 2026-09-24

### 🛠️ Fixed

- **(CHANGELOG)** partial runs of jj-release made CL out of order

## [1.2.1] - 2026-09-24

### ♻️ Changed

- **(preprocessor)** in process_book use let-else rather than if let + else
- **(book)** collapse process_chapter into strip_chapter_content
- **(frontmatter)** use borrowed fields rather than owned Strings in Frontmatter struct to reduce allocation
- **(frontmatter)** require both opening and closing --- in fenced_body_start, if no closing fence, treat as no frontmatter

### 🛠️ Fixed

- version
- **(frontmatter)** fix yaml extraction for fenced and unfenced frontmatter
- **(frontmatter)** bug in parse_frontmatter

## [1.2.0] - 2026-09-24

### ✨ Added

- **(examples)** add example demonstrating method use
- **(frontmatter)** add a few methods
- add examples/ dir with a short examples
- add custom Error enum and move away from anyhow
- add fuzzing with cargo-fuzz and arbitrary

### ♻️ Changed

- **(yaml_frontmatter)** make more lib friendly
- **(main)** replace anyhow with own Result type
- **(preprocessor)** replace anyhow with custom Result type

### 🛠️ Fixed

- **(preprocessor)** failing test still asserting on the old anyhow string
- **(error)** clippy lints
- **(frontmatter)** had 'n' instead of '\n' in normalize_body causing test to fail

### 🪲 Bug

- **(frontmatter)** replace remove(0) in normalize_body with trim_start_matches

## [1.1.3] - 2026-09-24

### ✨ Added

- add test coverage badge to README
- add badges to README

### ♻️ Changed

- **(cli)** const USAGE
- **(main)** remove redundant else block
- **(frontmatter)** slim down large function with helper functions

### 🛠️ Fixed

- mdBook uses items, not sections

## [1.1.2] - 2026-09-24

### ✨ Added

- add unit tests to main
- run cargo doc to generate documentation
- **(lib)** add inline unit tests to ensure behavior remains the same during refactor

### ♻️ Changed

- **(main)** create sub-modules and break up main
- **(lib)** create book module and move related functions there
- **(lib)** create frontmatter module and move strip_frontmatter there

### 🛠️ Fixed

- failing test because of incorrect logic
- clippy lints

## [1.1.1] - 2026-09-24

### ✨ Added

- fix clippy lints, update all dependencies

### 🛠️ Fixed

- remove unnecessary deps and upgrade deps

