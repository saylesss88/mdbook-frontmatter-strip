//! Command-line argument parsing and dispatch.
//!
//! `mdbook-frontmatter-strip` is invoked by `mdbook` itself (no args, or the
//! `supports <renderer>` capability probe), but also supports a couple of
//! manual flags (`--version`/`-V`) for humans checking what's installed.

use std::process;

/// What the binary was asked to do, based on the first CLI argument.
pub enum Command {
    /// mdBook's capability probe: `supports <renderer>`.
    Supports { renderer: String },
    /// `--version` or `-V`.
    Version,
    /// No arguments: normal mdBook preprocessing via stdin/stdout.
    Preprocess,
    /// Anything else: unrecognized, should print usage and exit non-zero.
    Unknown { arg: String },
}

/// Parse `std::env::args()` (already advanced past the program name) into a
/// [`Command`].
pub fn parse_args(mut args: impl Iterator<Item = String>) -> Command {
    match args.next().as_deref() {
        Some("supports") => Command::Supports {
            renderer: args.next().unwrap_or_default(),
        },
        Some("--version" | "-V") => Command::Version,
        None => Command::Preprocess,
        Some(other) => Command::Unknown {
            arg: other.to_string(),
        },
    }
}

/// Print `<name> <version>` using Cargo's built-in package metadata.
pub fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

/// Print usage information to stderr and exit with status 1.
pub fn print_usage_and_exit(unknown_arg: &str) -> ! {
    eprintln!("Unknown argument: {unknown_arg}");
    eprintln!("Usage:");
    eprintln!("  mdbook-frontmatter-strip                     # mdBook preprocessor");
    eprintln!("  mdbook-frontmatter-strip supports <renderer>");
    eprintln!("  mdbook-frontmatter-strip --version");
    process::exit(1);
}
