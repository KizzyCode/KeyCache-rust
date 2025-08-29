//! CLI utilities

use keycache::AuthLevel;
use std::env;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::path::{Path, PathBuf};

/// The filename to operate on
#[derive(Debug, Clone)]
pub struct Filename(PathBuf);
impl Filename {
    /// The file extension
    const EXTENSION: &str = ".keycache";
}
impl Deref for Filename {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<String> for Filename {
    fn from(filepath: String) -> Self {
        // See if the path needs post-processing
        let filepath = Path::new(&filepath);
        let false = filepath.exists() else {
            // If the path exists as-is, it is probably complete
            return Self(filepath.to_owned());
        };

        // Append extension if needed
        match filepath.extension() {
            None => Self(filepath.with_extension(Self::EXTENSION)),
            _ => Self(filepath.to_owned()),
        }
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
