# `keycache-libsecureenclave-sys`
This crate provides a sys-wrapper for <https://github.com/KizzyCode/secureenclave-c>.

## Bindgen
To regenerate the bindings after an update, call bindgen from the `libsecureenclave-sys` root directory:
```sh
bindgen --default-macro-constant-type=signed \
  --default-enum-style=rust \
  \
  --no-recursive-allowlist \
  --allowlist-type "sep_.*" \
  --allowlist-var "sep_.*" \
  --allowlist-function "sep_.*" \
  \
  --output ./src/ffi/libsecureenclave.rs ./libsecureenclave/Sources/SecureEnclave/include/SecureEnclave.h
```
