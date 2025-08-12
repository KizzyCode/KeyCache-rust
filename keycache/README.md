[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/Keycache-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/Keycache-rust)
[![docs.rs](https://docs.rs/keycache/badge.svg)](https://docs.rs/keycache)
[![crates.io](https://img.shields.io/crates/v/keycache.svg)](https://crates.io/crates/keycache)
[![Download numbers](https://img.shields.io/crates/d/keycache.svg)](https://crates.io/crates/keycache)
[![dependency status](https://deps.rs/crate/keycache/latest/status.svg)](https://deps.rs/crate/keycache)


# `keycache`
Welcome to `keycache` 🎉

`keycache` is an application that can cache passwords and other secret data in a semipermanent way, without exposing
them to the filesystem or similar. It works by sealing the key with the built-in secure element and storing the
encrypted, sealed keyfile instead.

`keycache` provides a quick-and-easy compromise that allows you to store and provide individual secrets in a
non-interactive way for scripts etc, or in a fast-interactive way via PIN or biometry; as opposed to plaintext files or
dedicated one-size-fits-all password manager setups.


## Authentication Levels
`keycache` provides different authentication levels when it comes to unlocking a sealed key:
- `unauthenticated`: Anyone, who has access to the sealed keyfile can unlock the key, if the secure element is available
  and cooperative.
- `interactive`: In addition to the sealed keyfile, the user must provide a PIN, or perform a similar interactive
  challenge to unseal the key.
- `biometry`: In addition to the sealed keyfile, the user must perform a biometric challenge to ensure that they
  themselves are physically present to unseal the key. Please note that for security reasons, implementations may
  invalidate cached keys if the enrolled biometry changes.


## Implementation
To seal a key, we generate an (indirectly) hardware-backed **key-encryption-key**, and seal the key with AES256-GCM and
a random nonce. The encrypted key is then stored in a file, together with the necessary metadata.


### Supported Platforms and Platform Specific Implementation: macOS
For macOS, we use the secure enclave to seal the key:
1. We request the SEP to generate a new hardware-backed P256 ECDH keypair with the correct access level.
   Currently we use `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly` for access level `unauthenticated`, 
   `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` with `kSecAccessControlUserPresence` for access level `interactive`,
   and `kSecAccessControlBiometryCurrentSet` for access level `biometry`.
2. We generate another random P256 ECDH keypair, and immediately drop the private key so we end up with a valid static
   public key only.
3. We ask the SEP to derive an ECDH shared secret with the hardware-backed private key, and the other static public key.
   The resulting shared secret is hashed with SHA256 – the resulting output is the **key-encryption-key**.


## Security Considerations
As always, security assessments depend on the attack scenarios they should protect against. Neither hardware security
elements, not biometry offer perfect security. There have been successful attacks against biometry sensors, as well as
secure elements to trick them or extract secret key material. Additionally, challenges like PIN entry are usually
susceptible to keyloggers or similar attacks, and thus cannot guarantee "true interactivity" either.

As always, if you have specific security requirements, it is mandatory that you evaluate the specific implementation and 
authentication level you intend to use against the potential attack scenarios you want to protect the secrets against.


## ⚠️ HAZMAT ⚠️ – Backups and Long Term Storage
Critically, as with other caches, `keycache` stored data is **not suited** for long-term or permanent storage. As the
secrets are encrypted with hardware-backed keys, they are effectively lost if the secure element refuses decryption. TPM
reset? – cache is invalidated. Unexpected change in system state? – cache is invalidated. New biometry enrolled? – cache
is invalidated. Hardware defect? – cache is invalidated. 

And while the cache should remain valid for most of the time during normal daily operations, you must remain aware that
supposedly small changes like software or firmware updates could invalidate the cache sometimes. Therefore, it is
**crucial** that you use `keycache` only – as the name implies – as cache, and not as long-term-storage solution.
Long-term secrets should always be stored somewhere else and then imported into `keycache`, so if the cache is
invalidated by whatever reason, you don't loose any data.
