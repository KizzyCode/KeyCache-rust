//! CLI utilities

use keycache::AuthLevel;
use std::env;
use std::io::{Error, ErrorKind};
use std::ops::Deref;

/// The filename to operate on
#[derive(Debug, Clone)]
pub struct Filename(String);
impl Deref for Filename {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<String> for Filename {
    fn from(name: String) -> Self {
        // Build sanitized keyfile name
        let filename: String = (name.bytes())
            .filter_map(|byte| match byte {
                // Filter/sanitize input characters
                byte if byte.is_ascii_alphanumeric() => Some(byte as char),
                b'.' | b'_' | b'-' => Some(byte as char),
                _ => None,
            })
            .take(128)
            // Append file extension
            .chain(".keycache".chars())
            .collect();
        Self(filename)
    }
}

/// CLI arguments
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Provided user authentication (e.g. PIN)
    pub flag_userauth: Option<String>,
    /// The authentication level (for key creation)
    pub flag_seal: Option<AuthLevel>,
    /// The keyfile name
    pub name: Filename,
}
impl CliArgs {
    /// Reads the given CLI arguments from the process environment
    pub fn from_env() -> Result<Self, Error> {
        // Read CLI arguments
        let mut flag_userauth = None;
        let mut flag_create = None;
        let mut name = None;
        for arg in env::args().skip(1) {
            match (arg.split_once('='), &flag_userauth, &flag_create, &name) {
                (Some(("--userauth", value)), None, _, _) => flag_userauth = Some(String::from(value)),
                (Some(("--seal", value)), _, None, _) => flag_create = Some(AuthLevel::try_from(value)?),
                (None, _, _, None) => name = Some(Filename::from(arg)),
                // The flag is either unknown, or the respective value has already been set
                _ => return Err(Error::new(ErrorKind::InvalidInput, format!("Unexpected argument: {arg}"))),
            }
        }

        // Init self
        let name = name.ok_or(Error::new(ErrorKind::InvalidInput, "Missing required argument: `name`"))?;
        Ok(Self { flag_userauth, flag_seal: flag_create, name })
    }
}
