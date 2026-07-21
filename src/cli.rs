//! Command-line argument parsing and dispatch.
//!
//! `mdbook-frontmatter-strip` is invoked by `mdbook` itself (no args, or the
//! `supports <renderer>` capability probe), but also supports a couple of
//! manual flags (`--version`/`-V`) for humans checking what's installed.

use std::process;

const USAGE: &str = "\
      mdbook-frontmatter-strip                     # mdBook preprocessor
      mdbook-frontmatter-strip supports <renderer>
      mdbook-frontmatter-strip --version
";

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
    eprintln!("Usage:\n{USAGE}");
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_is_preprocess() {
        let args: Vec<String> = vec![];
        assert!(matches!(parse_args(args.into_iter()), Command::Preprocess));
    }

    #[test]
    fn supports_html_captures_renderer() {
        let args = vec!["supports".to_string(), "html".to_string()];
        match parse_args(args.into_iter()) {
            Command::Supports { renderer } => assert_eq!(renderer, "html"),
            _ => panic!("expected Command::Supports"),
        }
    }

    #[test]
    fn supports_with_no_renderer_defaults_to_empty() {
        let args = vec!["supports".to_string()];
        match parse_args(args.into_iter()) {
            Command::Supports { renderer } => assert_eq!(renderer, ""),
            _ => panic!("expected Command::Supports"),
        }
    }

    #[test]
    fn dash_v_and_double_dash_version_both_recognized() {
        for flag in ["--version", "-V"] {
            let args = vec![flag.to_string()];
            assert!(matches!(parse_args(args.into_iter()), Command::Version));
        }
    }

    #[test]
    fn unrecognized_arg_is_unknown() {
        let args = vec!["frobnicate".to_string()];
        match parse_args(args.into_iter()) {
            Command::Unknown { arg } => assert_eq!(arg, "frobnicate"),
            _ => panic!("expected Command::Unknown"),
        }
    }
}
