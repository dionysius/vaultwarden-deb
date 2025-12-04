# Reopen

[![Actions Status](https://github.com/vorner/reopen/workflows/test/badge.svg)](https://github.com/vorner/reopen/actions)
[![codecov](https://codecov.io/gh/vorner/reopen/branch/master/graph/badge.svg?token=3KA3R2D9fV)](https://codecov.io/gh/vorner/reopen)
[![docs](https://docs.rs/reopen/badge.svg)](https://docs.rs/reopen)

A tiny `Read`/`Write` wrapper that can reopen the underlying IO object.

The main motivation is integration of logging with logrotate. Usually, when
logrotate wants to rotate log files, it moves the current log file to a new
place and creates a new empty file. However, for the new messages to appear in
the new file, a running program needs to close and reopen the file. This is
most often signalled by SIGHUP.

This allows reopening the IO object used inside the logging drain at runtime.

An example is in the [documentation](https://docs.rs/reopen).

## Future plans

The API feels feature complete to me, therefore there probably won't be much
happening here. But I'm still open to ideas what would be good to have or PRs
implementing it.

## Rustc version policy

The project will build on any rustc 1.31.0 or newer. The only exception is
feature flags added in the future, where enabling them might require newer
compiler.

The tests or examples don't have any particular version guarantee (future
versions of the project may only *build* on 1.31.0, but tests might require
never compiler).

Change to this policy would be considered an API breaking change and would
require bumping the version to 2.0.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.
