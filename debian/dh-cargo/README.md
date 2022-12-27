# dh-cargo fork fork

This is a slight fork of the debhelper script fork [dh-cargo-fork],
with the following functional differences since git commit c47f5559:

* Use `cargo` provided by `$PATH` so it could be provided by `rustup`
* Removed `-Z` in cargo install (did always print: *the `-Z` flag is only accepted on the nightly channel of Cargo, ...*)

[dh-cargo-fork]: <https://salsa.debian.org/build-common-team/dh-cargo-fork>
