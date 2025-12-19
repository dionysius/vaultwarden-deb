[![Workflow Status](https://github.com/fosskers/totp-lite/workflows/Rust/badge.svg)](https://github.com/fosskers/totp-lite/actions?query=workflow%3A%22Rust%22)
[![](https://img.shields.io/crates/v/totp-lite.svg)](https://crates.io/crates/totp-lite)

# totp-lite

A simple, correct TOTP library.

Time-based One-time Passwords are a useful way to authenticate a client,
since a valid password expires long before it could ever be guessed by an
attacker. This library provides an implementation of TOTP that matches its
specification [RFC6238], along with a simple interface.

## Usage

The `totp` function is likely what you need. It uses the default time step
of 30 seconds and produces 8 digits of output:

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp, Sha512};

// Negotiated between you and the authenticating service.
let password: &[u8] = b"secret";

// The number of seconds since the Unix Epoch.
let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

// Specify the desired Hash algorithm via a type parameter.
// `Sha1` and `Sha256` are also available.
let result: String = totp::<Sha512>(password, seconds);
assert_eq!(8, result.len());
```

For full control over how the algorithm is configured, consider
`totp_custom`.

## Resources
- [RFC6238: TOTP][RFC6238]
- [RFC6238 Errata](https://www.rfc-editor.org/errata_search.php?rfc=6238)

[RFC6238]: https://tools.ietf.org/html/rfc6238

License: MIT
