//! Entry point for the `mdbook-frontmatter-strip` binary.
//!
//! Dispatches based on how mdBook (or a human) invoked the binary:
//! - `supports <renderer>`: mdBook's capability probe.
//! - `--version` / `-V`: print version info and exit.
//! - no arguments: run as an mdBook preprocessor over stdin/stdout.
//! - anything else: print usage and exit with an error.

mod cli;
mod preprocessor;

use cli::Command;
use std::process;

/// Run the CLI; on failure, print the error's `Display` form and exit 1.
fn main() {
    if let Err(e) = run() {
        eprintln!("mdbook-frontmatter-strip: {e}");
        process::exit(1);
    }
}

/// Parse CLI arguments and dispatch to the appropriate behavior.
fn run() -> preprocessor::Result<()> {
    match cli::parse_args(std::env::args().skip(1)) {
        Command::Supports => Ok(()),
        Command::Version => {
            cli::print_version();
            Ok(())
        }

        Command::Preprocess => preprocessor::run_with_stdio(),
        Command::Unknown { arg } => cli::print_usage_and_exit(&arg),
    }
}
