//! Embeds the <https://github.com/KizzyCode/secureenclave-c> dylib

use crate::ffi::LibSecureEnclave;
use libloading::Library;
use std::env;
use std::ffi::{CStr, OsStr};
use std::fs::{self, File};
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::sync::LazyLock;

pub mod ffi;

/// The binary contents of the dylib for dynamic installation
pub const DYLIB_BIN: &[u8] = include_bytes!("../libsecureenclave/.build/release/libSecureEnclave-Dylib.dylib");

/// Loads the dylib and returns the associated handle
///
/// # Panic
/// This function panics if the dylib cannot be installed to a temporary location, or if the dylib cannot be loaded from
/// said location.
pub unsafe fn load() -> &'static LibSecureEnclave {
    /// Static lazy library handle
    static LIBRARY: LazyLock<Library> = LazyLock::new(|| {
        // Create tempfile
        let tempfile_path = env::temp_dir().join("libsecureenclave-XXXXXX.dylib\0");
        let mut tempfile_path = tempfile_path.into_os_string().into_vec();
        let tempfile_fd = unsafe { libc::mkstemps(tempfile_path.as_mut_ptr() as _, ".dylib".len() as _) };
        assert!(tempfile_fd > 0, "failed to create tempfile");

        // Write library to tempfile
        let mut tempfile = unsafe { File::from_raw_fd(tempfile_fd) };
        tempfile.write_all(DYLIB_BIN).expect("failed to write dylib to tempfile");

        // Convert the tempfile back into a usable path, and load dylib
        let tempfile_path =
            CStr::from_bytes_until_nul(&tempfile_path).expect("`mkstemps` assembled invalid tempfile path");
        let tempfile_path = OsStr::from_bytes(tempfile_path.to_bytes());
        let library = unsafe { Library::new(&tempfile_path).expect("failed to load dylib from tempfile") };

        // Unlink tempfile and return library handle
        fs::remove_file(tempfile_path).expect("failed to delete tempfile");
        library
    });

    // Load library and symbol wrapper
    unsafe { LibSecureEnclave::load(&LIBRARY) }
}
