# CodeMap

**[Documentation](https://docs.rs/codemap)**

[![Build Status](https://travis-ci.org/kevinmehall/codemap.svg?branch=master)](https://travis-ci.org/kevinmehall/codemap)

A data structure for tracking source positions in language implementations, inspired by the
[SourceMap (formerly CodeMap) type in rustc's libsyntax](https://github.com/rust-lang/rust/blob/master/src/libsyntax/source_map.rs).

The `CodeMap` tracks all source files and maps positions within them to linear indexes as if all
source files were concatenated. This allows a source position to be represented by a small
32-bit `Pos` indexing into the `CodeMap`, under the assumption that the total amount of parsed
source code will not exceed 4GiB. The `CodeMap` can look up the source file, line, and column
of a `Pos` or `Span`, as well as provide source code snippets for error reporting.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
