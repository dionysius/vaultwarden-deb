# `diesel-derive-newtype`

Easy-peasy support of newtypes inside of Diesel.

[![Rust](https://github.com/quodlibetor/diesel-derive-newtype/actions/workflows/test.yml/badge.svg?branch=diesel-2)](https://github.com/quodlibetor/diesel-derive-newtype/actions/workflows/test.yml) [![Crates.io Version](https://img.shields.io/crates/v/diesel-derive-newtype.svg)](https://crates.io/crates/diesel-derive-newtype)

## Installation

diesel-derive-newtype supports Diesel according to its major version -- 0.x
through 1.x support the corresponding diesel versions, 2.0 supports diesel 2.0,
and 2.1 supports 2.1.

New features are only developed for the currently supported version of Diesel.


```toml
[dependencies]
diesel-derive-newtype = "2.1.0"
```

for Diesel 2.0.x you have to tell cargo not to upgrade diesel-derive-newtype:

```toml
[dependencies]
diesel-derive-newtype = "~ 2.0.0"
```

And for Diesel 1.x:

```toml
[dependencies]
diesel-derive-newtype = "1.0"
```



## `#[derive(DieselNewType)]`

This crate exposes a single custom-derive macro `DieselNewType` which
implements `ToSql`, `FromSql`, `FromSqlRow`, `Queryable`, `AsExpression`
and `QueryId` for the single-field tuple struct ([NewType][]) it is applied
to.

The goal of this project is that:

* `derive(DieselNewType)` should be enough for you to use newtypes anywhere you
  would use their underlying types within Diesel. (plausibly successful)
* Should get the same compile-time guarantees when using your newtypes as
  expression elements in Diesel as you do in other rust code (depends on
  your desires, see [Limitations][], below.)

[NewType]: https://aturon.github.io/features/types/newtype.html

## Example

This implementation:

```rust
#[macro_use]
extern crate diesel_derive_newtype;

#[derive(DieselNewType)] // Doesn't need to be on its own line
#[derive(Debug, Hash, PartialEq, Eq)] // required by diesel
struct MyId(i64);
```

Allows you to use the `MyId` struct inside your entities as though they were
the underlying type:

```rust
table! {
    my_items {
        id -> Integer,
        val -> Integer,
    }
}

#[derive(Debug, PartialEq, Identifiable, Queryable)]
struct MyItem {
    id: MyId,
    val: u8,
}
```

Oooohhh. Ahhhh.

See [the tests][] for a more complete example.

[the tests]: https://github.com/quodlibetor/diesel-derive-newtype/blob/master/tests/db-roundtrips.rs

## Limitations
[limitations]: #limitations

The `DieselNewtype` derive does not create new _database_ types, or Diesel
serialization types. That is, if you have a `MyId(i64)`, this will use
Diesel's underlying `BigInt` type, which means that even though your
newtypes can be used anywhere the underlying type can be used, *the
underlying types, or any other newtypes of the same underlying type, can be
used as well*.

At a certain point everything does become bits on the wire, so if we didn't
do it this way then Diesel would have to do it somewhere else, and this is
reasonable default behavior (it's pretty debuggable), but I'm investigating
auto-generating new proxy types as well to make it impossible to construct
an insert statement using a tuple or a mis-typed struct.

Here's an example of that this type-hole looks like:

```rust
#[derive(Debug, Hash, PartialEq, Eq, DieselNewType)]
struct OneId(i64);

#[derive(Debug, Hash, PartialEq, Eq, DieselNewType)]
struct OtherId(i64);

#[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
#[diesel(table_name = my_entities)]
pub struct MyEntity {
    id: OneId,
    val: i32,
}

fn darn(conn: &Connection) {
    // shouldn't allow constructing the wrong type, but does
    let OtherId: Vec<OtherId> = my_entities
        .select(id)
        .filter(id.eq(OtherId(1)))  // shouldn't allow filtering by wrong type
        .execute(conn).unwrap();
}
```

See [`tests/should-not-compile.rs`](tests/should-not-compile.rs) for the
things I think should fail to compile.

I believe that the root cause of this is that Diesel implements the various
expression methods for types that implement `AsExpression`, based on the
_SQL_ type, not caring about `self` and `other`'s Rust type matching. That
seems like a pretty good decision in general, but it is a bit unfortunate
here.

I hope to find a solution that doesn't involve implementing every
`*Expression` trait manually with an extra bound, but for now you have to
keep in mind that the Diesel methods basically auto-transmute your data into
the underlying SQL type.


## Releasing

This workflow uses:

* [cargo-readme](https://crates.io/crates/cargo-readme)
* [cargo-release](https://crates.io/crates/cargo-release)

Run, note that we always release patch releases unless diesel has got a new
release:

```
cargo readme > README.md
git diff --exit-code --quiet README.* || (git add README.* && git commit -m "chore: Update README")
cargo release patch
```

## License

diesel-derive-newtype is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Patches and bug reports welcome!
