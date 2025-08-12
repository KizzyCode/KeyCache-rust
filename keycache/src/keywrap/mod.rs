//! Platform-agnostic keywrap implementation

#[cfg_attr(target_os = "macos", path = "macos.rs")]
mod platform_impl;

use aes_gcm::aead::AeadMutInPlace;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{Aes256Gcm, KeyInit};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

/// The authentication level necessary to unseal the key
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum AuthLevel {
    /// Anyone, who has access to the sealed keyfile can unlock the secret, if the secure element is available and
    /// cooperative
    Unauthenticated,
    /// In addition to the sealed keyfile, the user must provide a PIN, or perform a similar interactive challenge to
    /// unseal the secret
    Interactive,
    /// In addition to the sealed keyfile, the user must perform a biometric challenge to ensure that they themselves
    /// are physically present to unseal the secret
    Biometry,
}
impl TryFrom<&str> for AuthLevel {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "unauthenticated" => Ok(Self::Unauthenticated),
            "interactive" => Ok(Self::Interactive),
            "biometry" => Ok(Self::Biometry),
            _ => Err(Error::new(ErrorKind::InvalidInput, format!("Invalid authentication level: {value}"))),
        }
    }
}

/// A sealed key ciphertext
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SealedKey {
    /// The nonce the key was sealed with
    #[serde(with = "serde_bytes")]
    nonce: [u8; 12],
    /// The sealed key itself
    #[serde(with = "serde_bytes")]
    ciphertext: Vec<u8>,
    /// The associated authentication tag
    #[serde(with = "serde_bytes")]
    mac: [u8; 16],
}

/// Creates a new hardware-backed key-encryption-key and returns the necessary metadata to use the key later
pub fn create(auth_level: AuthLevel) -> Result<Vec<u8>, Error> {
    platform_impl::create(auth_level)
}

/// Unlocks a key-encryption-key via the associated metadata
///
/// ## `user_auth` required
/// As a hint that decryption might succeed if user authentication is provided, implementations _might_ return
/// [`ErrorKind::InvalidInput`] as I/O error kind.
pub fn unlock(metadata: &[u8], user_auth: Option<&[u8]>) -> Result<[u8; 32], Error> {
    platform_impl::unlock(metadata, user_auth)
}

/// Wraps a key with the given key-encryption-key and AES256-GCM
pub fn wrap(key: &[u8], key_encryption_key: &[u8; 32]) -> Result<Vec<u8>, Error> {
    // Generate random nonce
    let mut nonce = [0; 12];
    getrandom::fill(&mut nonce)?;

    // Initialize cipher
    let nonce = GenericArray::from_slice(&nonce);
    let key_encryption_key_ = GenericArray::from_slice(key_encryption_key);
    let mut aes256_gcm = Aes256Gcm::new(key_encryption_key_);

    // Seal key
    let mut key = key.to_vec();
    let mac = aes256_gcm.encrypt_in_place_detached(nonce, b"", &mut key).expect("failed to seal key");

    // Assemble ciphertext
    let sealed_key = SealedKey { nonce: (*nonce).into(), ciphertext: key, mac: mac.into() };
    let sealed_key = serde_asn1_der::to_vec(&sealed_key).expect("failed to serialize ciphertext");
    Ok(sealed_key)
}

/// Unwraps a key with the given key-encryption-key and AES
pub fn unwrap(sealed_key: &[u8], key_encryption_key: &[u8; 32]) -> Result<Vec<u8>, Error> {
    // Parse ciphertext
    let SealedKey { nonce, ciphertext: mut chiphertext, mac } =
        serde_asn1_der::from_bytes(&sealed_key).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    let nonce = GenericArray::from_slice(&nonce);
    let mac = GenericArray::from_slice(&mac);

    // Open key
    let key_encryption_key = GenericArray::from_slice(key_encryption_key);
    let mut aes256_gcm = Aes256Gcm::new(key_encryption_key);
    (aes256_gcm.decrypt_in_place_detached(nonce, b"", &mut chiphertext, mac))
        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
    Ok(chiphertext)
}
