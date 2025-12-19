# 1.0.3

* Update signal-hook dependency to 0.3.

# 1.0.2

* Update signal-hook dependency to 0.2.
  Note: this is *not* breaking change. While we do re-export the `SigId`, that
  one comes from `signal-hook-registry`, which is already 1.0. So it is the same
  type.

# 1.0.1

* Fix feature specification, so the `signals` feature compiles.

# 1.0.0

* Allow user code to lock against reopening for a while.
* Don't interrupt bulk operations (`write_all`, `read_to_string`) by reopening,
  check reopening only once before they start.
* Reopen implements Debug
* `read_vectored` and `write_vectored` support where already provided by `std`
* The signal support is enabled by a feature (default off)
  - Windows now has it too
* Migrated to edition 2018, fixed the low Rust version to 1.31.0

# 0.3.0

* Delegated the signal handling to the signal-hook crate, so the same signal can
  be shared with other things and the code in this crate is simplified (breaking
  change in the `register_signal` method return type).
* Fixed a bug with extra reopen just after creation.

# 0.2.1

* Lifted the many annoying limitations of `Handle::register_signal`.
* Made the `Handle::register_signal` function safe.
* Added an example.

# 0.2.0

* Minor fixes in documentation links.
* Error handling improvements:
  - Better documentation for what happens.
  - Perform first opening in the constructor, getting a potential serious error
    on the first try.

# Older versions

* ?? No historical records…
