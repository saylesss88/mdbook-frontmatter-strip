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

use mdbook_frontmatter_strip::error::Result;

/// Parse CLI arguments and dispatch to the appropriate behavior.
fn main() -> Result<()> {
    match cli::parse_args(std::env::args().skip(1)) {
        Command::Supports { renderer } => {
            if renderer == "html" {
                process::exit(0);
            }
            process::exit(1);
        }
        Command::Version => {
            cli::print_version();
            Ok(())
        }

        Command::Preprocess => preprocessor::run_with_stdio(),
        Command::Unknown { arg } => cli::print_usage_and_exit(&arg),
    }
}
