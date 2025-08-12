use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    // Assemble target dir path for build artifacts
    let target_dir = env::var("OUT_DIR").expect("failed to get `OUT_DIR`");
    let swift_build_dir = Path::new(&target_dir).join(".swift-build");
    let dylib_path = swift_build_dir.join("release").join("libSecureEnclave-Dylib.dylib");

    // Convert paths to string
    let swift_build_dir = swift_build_dir.to_str().expect("build path contains non-utf8-chars");
    let dylib_path = dylib_path.to_str().expect("build path contains non-utf8-chars");

    // Build dylib
    let output = Command::new("swift")
        .args(["build", "--configuration", "release", "--scratch-path", &swift_build_dir])
        .current_dir("libsecureenclave")
        .output()
        .expect("failed to build libsecureenclave");
    assert!(output.status.success(), "failed to build libsecureenclave: {}", String::from_utf8_lossy(&output.stderr));

    // Ensure the dylib is always built, and forward the dylib artifact path
    println!("cargo:rerun-if-changed={dylib_path}");
    println!("cargo:rustc-env=LIBSECUREENCLAVE_DYLIB_PATH={dylib_path}");
}
