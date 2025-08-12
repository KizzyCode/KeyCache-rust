#![doc = include_str!("../README.md")]
// Clippy lints
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unreachable)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::allow_attributes_without_reason)]
#![warn(clippy::cognitive_complexity)]

mod cli;

use crate::cli::CliArgs;
use std::io::{self, Error, Read, Write};
use std::{fs, process};

/// Display the help and exit with `1`
fn abort(error: Error) -> ! {
    /// Usage instructions
    const USAGE: &str = include_str!("../USAGE.txt");

    // Print error and usage
    eprintln!("{error}");
    eprintln!("-----");
    eprintln!("");
    eprintln!("{USAGE}");
    process::exit(1);
}

/// Fallible main application logic
fn application() -> Result<(), Error> {
    // Read CLI arguments
    let CliArgs { flag_userauth, flag_seal: flag_create, name } = CliArgs::from_env()?;
    let user_auth = flag_userauth.as_ref().map(|userauth| userauth.as_bytes());

    // Execute command
    match flag_create {
        Some(flag_create) => {
            // Read key from stdin and seal it
            let mut key = Vec::new();
            io::stdin().read_to_end(&mut key)?;
            let sealed_key = keycache::seal(&key, flag_create, user_auth)?;

            // Write sealed key as `name`
            fs::write(name.as_str(), sealed_key)?;
            Ok(())
        }
        None => {
            // Read sealed key from file and open it
            let sealed_key = fs::read(name.as_str())?;
            let key = keycache::open(&sealed_key, user_auth)?;

            // Write key to stdout
            io::stdout().write_all(&key)?;
            Ok(())
        }
    }
}

pub fn main() {
    // Run application logic, or display error and usage
    let _ = application().map_err(abort);
}
