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

mod keywrap;

pub use crate::keywrap::AuthLevel;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

/// A sealed cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// The necessary metadata to use the hardware key later
    #[serde(with = "serde_bytes")]
    metadata: Vec<u8>,
    /// The sealed user key data
    #[serde(with = "serde_bytes")]
    sealed_key: Vec<u8>,
}

/// Seals a key with the given authentication level
///
/// ## `user_auth` required
/// As a hint that decryption might succeed if user authentication is provided, implementations _might_ return
/// [`ErrorKind::InvalidInput`] as I/O error kind.
pub fn seal(key: &[u8], auth_level: AuthLevel, user_auth: Option<&[u8]>) -> Result<Vec<u8>, Error> {
    // Creates a new hardware-backed key, derives the key-encryption-key and seals the user key data
    let metadata = keywrap::create(auth_level)?;
    let key_encryption_key = keywrap::unlock(&metadata, user_auth)?;
    let sealed_key = keywrap::wrap(key, &key_encryption_key)?;

    // Assemble and serialize the cache entry
    let cache_entry = CacheEntry { metadata, sealed_key };
    let cache_entry = serde_asn1_der::to_vec(&cache_entry).expect("failed to serialize cache entry");
    Ok(cache_entry)
}

/// Opens a sealed key
///
/// ## `user_auth` required
/// As a hint that decryption might succeed if user authentication is provided, implementations _might_ return
/// [`ErrorKind::InvalidInput`] as I/O error kind.
pub fn open(sealed_key: &[u8], user_auth: Option<&[u8]>) -> Result<Vec<u8>, Error> {
    // Parse the cache entry
    let CacheEntry { metadata, sealed_key } =
        serde_asn1_der::from_bytes(&sealed_key).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    // Unlock the key-encryption-key and open the user key data
    let key_encryption_key = keywrap::unlock(&metadata, user_auth)?;
    let key = keywrap::unwrap(&sealed_key, &key_encryption_key)?;
    Ok(key)
}
