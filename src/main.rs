mod cli;
mod preprocessor;

use anyhow::Result;
use cli::Command;
use std::process;

fn main() -> Result<()> {
    match cli::parse_args(std::env::args().skip(1)) {
        Command::Supports { renderer } => {
            if renderer == "html" {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        Command::Version => {
            cli::print_version();
            Ok(())
        }

        Command::Preprocess => preprocessor::run_with_stdio(),
        Command::Unknown { arg } => cli::print_usage_and_exit(&arg),
    }
}
