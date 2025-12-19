# Change Log
All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

### pq-sys [0.7.2] 2025-06-12

* Bump bindgen to 0.72.0 and regenerate bindings
* Fixed an issue with bundled builds without openssl on windows due to a breaking upstream rust change 

### pq-src [0.3.8] 2025-05-10

* Fixed libpq version specifier and yanked the 0.3.7 release

### pq-src [0.3.7] 2025-05-09

* Fix builds on alpine linux
* Update the bundled libpq version to 17.5

### pq-src [0.3.6] 2025-04-04

* Fix broken builds on MacOS 15.4 by checking for the newly added `strchrnul` function there

### pq-sys [0.7.1] 2025-03-27

* Restore compablilities with non-cargo build systems
* Add a safe-guard to make sure users get a better error message if the build script is not run

### pq-src [0.3.5] 2025-03-17

* Update the bundled libpq version to 17.4

### pq-src [0.3.4] 2025-03-03

* Include three missing header files for the without openssl configuration

### pq-src [0.3.3] 2025-01-30

* Remove scripts and other unused files from the published package

### pq-sys [0.7.0] 2025-01-10

* Bump bindgen to 0.71
* Regenerate bindings and make sure that they compile on common platforms
* Remove non-libpq items from the generated bindings
* Bump crate to 2021 edition
* Increase minimal supported rust version to 1.82

### pq-src [0.3.2] 2024-09-27

### Changed

* Update the bundled libpq version to the postgres 17.0 release

### pq-sys [0.6.3] 2024-09-18

### Fixed

* Linking of bundled libpq with the `bundled_without_openssl` feature

### pq-sys [0.6.2] 2024-09-09

### Changed

* `openssl` usage can now be disabled with the `bundled_without_openssl` feature of `pq-sys`.
  To deactivate `openssl`, enable it:
  ```toml
  [dependencies]
  pq-sys = { version = "0.3.0", features = ["bundled_without_openssl"]}
  ```
  This disables the TLS support in `libpq`, so that you cannot connect to a database requiring TLS anymore.

## pq-sys [0.6.1] 2024-06-11

### Changed

* `pq-src` starting with version 0.3 now depends on openssl on windows as well add support for the scram-sha-256 authentication method
* `pq-sys` adds support for pq-src 0.3.0

## pq-sys [0.6.0] 2024-05-17

### Changed

* The `bundled` feature of `pq-sys` no longer build a bundled version of `openssl`.
  To bundle `openssl` when compiling `libpq` from source, add `openssl-sys` with the `vendored` feature to your crate dependencies:
  ```toml
  [dependencies]
  openssl-sys = { version = "0.9.93", features = ["vendored"] }
  ```

## pq-src [0.1.6] 2024-04-12

### Changed

* Fixed a potential compilation error on MacOS

## pq-src [0.1.5] 2024-04-10

### Changed

* Include a vendored copy of `explicit_bzero` to the linux build as well as that function only exists on newer glibc versions

## pq-src [0.1.4] 2024-04-05

### Changed

* Readded `strlcat` on linux as well

## pq-src [0.1.3] 2024-04-04

### Changed

* Include more required headers in the generated include directory
* Fix including `strlcat` on windows only

## pq-src [0.1.2] 2024-03-08

### Changed

* Change `#[cfg(target_*)]` attributes in the build script to use cargo set environment variables
  instead. This enables cross compiling `pq-src` to a different target system.

## pq-src [0.1.1] 2024-02-09

### Changed

- We updated the bundled libpq version to 16.2 to fix compatibility with openssl 3.2
- We changed the build script to expose the bundled libpq headers by setting `DEP_PQ_SYS_SRC_INCLUDE` so that other *-sys crates can consume that information and reuse the bundled build provided by this crate


## [0.5.0] 2024-01-19

### Added

- We added a `pq-src` crate and a `bundled` feature for `pq-sys`. This allows to build and link a static version of libpq during the rust build process. This feature currently supports builds targeting Windows, Linux and macOS. It requires a c-compiler toolchain for the target to build libpq from source.
- We added a `buildtime_bindgen` feature flag that allows to generate bindings for your locally installed libpq version. This is helpful for cases where the target architecture is significantly different to what the built-in bindings assume.

### Changed

- We regenerated the bundled bindings to match the libpq version build by the `bundled` feature flag

## [0.4.8] 2023-04-18

## Fixed

- Added `wldap` as dependency for the vcpk installation on windows as that's now required there

## [0.4.5] 2018-05-09

### Added

- Linking can now be dynamically handled per-target by specifying the `TARGET`
  environment variable and `PQ_LIB_STATIC_$TARGET` environment variable.

- The path to `pg_config` can now be dynamically handled per-target by
  specifying the `TARGET` environment variable and `PG_CONFIG_$TARGET`
  environment variable.

## [0.4.3] 2017-03-10

### Fixed

- Linking on the msvc toolchain will no longer attempt to statically link

## [0.4.2] 2017-02-19

### Fixed

- Improved linking on Windows, particularly with the msvc toolchain.

## [0.4.1] 2017-02-19

### Fixed

- Properly specified the build script

## [0.4.0] 2017-02-19 [YANKED]

### Changed

- Bindings are no longer generated at compile time. Requiring clang 3.9 caused
  too many issues for too many users, and requiring `syntex_syntax` increased
  compile time too much.

## [0.3.2] 2017-02-16

### Fixed

- Fixed an issue when building on mac against postgres from homebrew

## [0.3.1] 2017-02-16

### Fixed

- Fixed an issue when building on mac against postgres from homebrew

- Fixed an issue with locating header files on linux

## [0.3.0] 2017-02-16

### Changed

- Bindings are now generated when the library is built, rather than being
  vendored ahead of time.

- `libc` is no longer used. Anywhere that `libc::some_type` was expected,
  `std::os::raw::some_type` is now used instead.

- The build script will no longer attempt to canonicalize symlinks on MacOS.

## [0.2.7] 2016-12-10

### Changed

- `pkg-config` is disabled by default. It can be enabled by adding `features =
  ["pkg-config"]` to your `Cargo.toml`.

## [0.2.6] 2016-12-10

### Added

- We will attempt to use `pkg-config` to locate libpq before falling back to
  `pg_config`.

## [0.2.5] 2016-12-10

- No changes. Accidental release on the wrong commit.

## [0.2.4] 2016-11-22

### Added

- `pq` will be statically linked if the environment variable `PQ_LIB_STATIC` is
  set.

## [0.2.3] 2016-08-12

### Changed

- On Mac if `pg_config` points to a directory where `libpq.dylib` is a symlink,
  we will now attempt to find the canonical directory to link against. This
  means that for installations using homebrew,
  `/usr/local/Cellar/postgresql/version/lib` will be added to `DYLD_LIBRARY_PATH`
  instead of `/usr/local/lib`

## [0.2.2] 2016-07-30

### Added

- The directory containing libpq for linking can now be specified via the
  `PQ_LIB_DIR` environment variable.
