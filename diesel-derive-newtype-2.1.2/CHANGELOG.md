# Unreleased

# Version 2.1.2

* Fix new non_local_definitions lint in nightly (#31)

# Version 2.1.1

* Add support for structs with internal references to DieselNewTypes (`ethan-lowman-fp` [#30](https://github.com/quodlibetor/diesel-derive-newtype/pull/30)):

  ```rust
  #[derive(DieselNewType)]
  pub struct MyIdString(String); 
 
  #[derive(Insertable, Queryable)]
  #[diesel(table_name = my_entities)]
  pub struct NewMyEntity<'a> {
      id: &'a MyIdString,  // <-- &'a of DieselNewType
  }
  ```

# 2.1.0

* Update for Diesel 2.1 (`@marhag87`), not compatible with Diesel 2.0.x.
* Bump MSRV to 1.65, because that is Diesel's MSRV.

# 2.0.1

* Bind diesel-derive-newtype 2.0.x to Diesel 2.0.x, Diesel 2.1 has trait bounds that are
  non-obvious to make compatible with 2.0.

# 2.0.0

* Support diesel 2.0. diesel-derive-newtype v1.* supports Diesel 1.* and v2.* supports Diesel 2.*

# 1.0.2

* Update syn to 2.0, bump MSRV to 1.56

# 1.0.1

* Update syn/quote/proc-macro2 dependencies to 1.x

# 1.0.0

* Remove non-dev dependency on `diesel` -- `diesel-derive-newtype` generates generic diesel code.
* CI improvements.

# 0.1.1

Bugs Fixed:

* Issue #5: Deriving NewType in the same module as an unnamespaced result
  caused problems. Report and fix by @adwhit


# 0.1.0

Initial release
