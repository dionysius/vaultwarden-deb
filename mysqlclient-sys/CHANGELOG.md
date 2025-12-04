# Change Log
All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## [0.4.7] 2025-06-17

* Update the bundled mysqlclient source version to 0.9.3
* Add pregenerated bindings for mysqlclient 9.3.x
* Reenable the `mysql_native_password` plugin
* Improved some error messages if no matching bindings where found for the specified mysqlclient version

## [0.4.6] 2025-06-10

* Update bindgen version to 0.72.0

## [0.4.5] 2025-04-08

* Fix libmariadb bindings version mapping

## [0.4.4] 2025-04-04

* Unconditionally link to advapi32 on windows to fix the linking problem in more cases

## [0.4.3] 2025-03-28

* Adding a link annotation for advapi32 on windows as the rust-standard library stops linking it
* Updated the bindgen dependency to 0.71 and regenerated all bindings
* Updated the bundled libmysqlclient version to 9.2.0

## [0.4.2] 2024-11-22

## Added

* Trigger rebuild on pkg-config probed mysql library version bump 

## [0.4.1] 2024-08-23

## Added

* Added support for libmysqlclient 9.0.x
* Updated the bundled libmysqlclient version to 9.0.1

## [0.4.0] 2024-06-13

## Changed 

* Added support and tests for linking libmariadb on all platforms
* Refactor handling of pregenerated bindings to provide bindings for more platforms out of the box
* Fixed a bug that prevented using the `buildtime_bindgen` on non x86_64 architectures

## [0.3.1] 2024-05-31

### Changed

* Include a notice in the readme that the mysqlclient-src crate is licenced under GPL-v2
* Excluded more files from the mysql source code to minimize the size of mysqlclient-src
* Included the debian/ubuntu version specifier for libmariadb-dev to fix selecting the right bindings in the offical rust docker images

## [0.3.0] 2024-05-17

### Added

- We added a `mysqlclient-src` crate and a `bundled` feature for `mysqlclient-sys`. This allows to build and link a static version of libmysqlclient during the rust build process. This feature currently supports targeting Windows, Linux and macOS. It requires a c++ compiler toolchain and cmake to build libmysqlclient from source.
- We added a `buildtime_bindgen` feature flag that allows to generate bindings for your locally installed libmysqlclient version. This is helpful for cases where the target architecture is significantly different to what the built-in bindings assume.


### Changed

- We regenerated the bundled bindings for several libmysqlclient versions. You might now need to set the `MYSQLCLIENT_VERSION` environment to select the matching bindings for your libmysqlclient version
