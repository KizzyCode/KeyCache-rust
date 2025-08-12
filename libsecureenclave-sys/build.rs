use std::fs;
use std::process::Command;

/// `CACHEDIR.TAG` contents
const CACHEDIR_TAG: &str = concat! {
    "Signature: 8a477f597d28d172789f06886806bc55\n",
    "# This file is a cache directory tag created by cargo.\n",
    "# For information about cache directory tags see https://bford.info/cachedir/\n"
};

fn main() {
    // Build dylib
    let output = Command::new("swift")
        .args(["build", "--configuration", "release"])
        .current_dir("libsecureenclave")
        .output()
        .expect("failed to build libsecureenclave");
    assert!(output.status.success(), "failed to build libsecureenclave: {}", String::from_utf8_lossy(&output.stderr));

    // Add cachedir-tag
    fs::write("./libsecureenclave/.build/CACHEDIR.TAG", CACHEDIR_TAG)
        .expect("failed to add cachedir-tag into build directory");

    // Ensure the dylib is always built
    println!("cargo:rerun-if-changed=./libsecureenclave/.build/release/libSecureEnclave-Dylib.dylib");
}
