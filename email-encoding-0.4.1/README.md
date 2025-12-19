# email-encoding

[![crates.io](https://img.shields.io/crates/v/email-encoding.svg)](https://crates.io/crates/email-encoding)
[![Documentation](https://docs.rs/email-encoding/badge.svg)](https://docs.rs/email-encoding)
[![dependency status](https://deps.rs/crate/email-encoding/0.3.1/status.svg)](https://deps.rs/crate/email-encoding/0.3.1)
[![Rustc Version 1.63.0+](https://img.shields.io/badge/rustc-1.63.0+-lightgray.svg)](https://blog.rust-lang.org/2022/08/11/Rust-1.63.0.html)
[![CI](https://github.com/lettre/email-encoding/actions/workflows/ci.yml/badge.svg)](https://github.com/lettre/email-encoding/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/lettre/email-encoding/branch/main/graph/badge.svg)](https://codecov.io/gh/lettre/email-encoding)

Low-level `#[no_std]` crate implementing various RFCs for encoding emails.
Used internally by [lettre].

Implemented RFCs:

* [RFC 2047]
* [RFC 2231]

[lettre]: https://crates.io/crates/lettre
[RFC 2047]: https://datatracker.ietf.org/doc/html/rfc2047
[RFC 2231]: https://datatracker.ietf.org/doc/html/rfc2231
