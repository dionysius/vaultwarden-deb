#![recursion_limit = "1024"] // the new default in rust 1.19, quote! takes a lot

//! # `#[derive(DieselNewType)]`
//!
//! This crate exposes a single custom-derive macro `DieselNewType` which
//! implements `ToSql`, `FromSql`, `FromSqlRow`, `Queryable`, `AsExpression`
//! and `QueryId` for the single-field tuple struct ([NewType][]) it is applied
//! to.
//!
//! The goal of this project is that:
//!
//! * `derive(DieselNewType)` should be enough for you to use newtypes anywhere you
//!   would use their underlying types within Diesel. (plausibly successful)
//! * Should get the same compile-time guarantees when using your newtypes as
//!   expression elements in Diesel as you do in other rust code (depends on
//!   your desires, see [Limitations][], below.)
//!
//! [NewType]: https://aturon.github.io/features/types/newtype.html
//!
//! # Example
//!
//! This implementation:
//!
//! ```
//! #[macro_use]
//! extern crate diesel_derive_newtype;
//!
//! #[derive(DieselNewType)] // Doesn't need to be on its own line
//! #[derive(Debug, Hash, PartialEq, Eq)] // required by diesel
//! struct MyId(i64);
//! # fn main() {}
//! ```
//!
//! Allows you to use the `MyId` struct inside your entities as though they were
//! the underlying type:
//!
//! ```
//! # #[macro_use] extern crate diesel;
//! # #[macro_use] extern crate diesel_derive_newtype;
//! # use diesel::prelude::*;
//! table! {
//!     my_items {
//!         id -> Integer,
//!         val -> Integer,
//!     }
//! }
//!
//! # #[derive(DieselNewType)] // Doesn't need to be on its own line
//! # #[derive(Debug, Hash, PartialEq, Eq)] // required by diesel
//! # struct MyId(i64);
//! #[derive(Debug, PartialEq, Identifiable, Queryable)]
//! struct MyItem {
//!     id: MyId,
//!     val: u8,
//! }
//! # fn main() {}
//! ```
//!
//! Oooohhh. Ahhhh.
//!
//! See [the tests][] for a more complete example.
//!
//! [the tests]: https://github.com/quodlibetor/diesel-derive-newtype/blob/master/tests/db-roundtrips.rs
//!
//! # Limitations
//! [limitations]: #limitations
//!
//! The `DieselNewtype` derive does not create new _database_ types, or Diesel
//! serialization types. That is, if you have a `MyId(i64)`, this will use
//! Diesel's underlying `BigInt` type, which means that even though your
//! newtypes can be used anywhere the underlying type can be used, *the
//! underlying types, or any other newtypes of the same underlying type, can be
//! used as well*.
//!
//! At a certain point everything does become bits on the wire, so if we didn't
//! do it this way then Diesel would have to do it somewhere else, and this is
//! reasonable default behavior (it's pretty debuggable), but I'm investigating
//! auto-generating new proxy types as well to make it impossible to construct
//! an insert statement using a tuple or a mis-typed struct.
//!
//! Here's an example of that this type-hole looks like:
//!
//! ```rust,ignore
//! #[derive(Debug, Hash, PartialEq, Eq, DieselNewType)]
//! struct OneId(i64);
//!
//! #[derive(Debug, Hash, PartialEq, Eq, DieselNewType)]
//! struct OtherId(i64);
//!
//! #[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
//! #[diesel(table_name = my_entities)]
//! pub struct MyEntity {
//!     id: OneId,
//!     val: i32,
//! }
//!
//! fn darn(conn: &Connection) {
//!     // shouldn't allow constructing the wrong type, but does
//!     let OtherId: Vec<OtherId> = my_entities
//!         .select(id)
//!         .filter(id.eq(OtherId(1)))  // shouldn't allow filtering by wrong type
//!         .execute(conn).unwrap();
//! }
//! ```
//!
//! See [`tests/should-not-compile.rs`](tests/should-not-compile.rs) for the
//! things I think should fail to compile.
//!
//! I believe that the root cause of this is that Diesel implements the various
//! expression methods for types that implement `AsExpression`, based on the
//! _SQL_ type, not caring about `self` and `other`'s Rust type matching. That
//! seems like a pretty good decision in general, but it is a bit unfortunate
//! here.
//!
//! I hope to find a solution that doesn't involve implementing every
//! `*Expression` trait manually with an extra bound, but for now you have to
//! keep in mind that the Diesel methods basically auto-transmute your data into
//! the underlying SQL type.

extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::{Span, TokenStream};

#[proc_macro_derive(DieselNewType)]
#[doc(hidden)]
pub fn diesel_new_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    expand_sql_types(&ast).into()
}

fn expand_sql_types(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let wrapped_ty = match ast.data {
        syn::Data::Struct(ref data) => {
            let mut iter = data.fields.iter();
            match (iter.next(), iter.next()) {
                (Some(field), None) => &field.ty,
                (_, _) => panic!(
                    "#[derive(DieselNewType)] can only be used with structs with exactly one field"
                ),
            }
        }
        _ => panic!("#[derive(DieselNewType)] can only be used with structs with a single field"),
    };

    // Required to be able to insert/read from the db, don't allow searching
    let to_sql_impl = gen_tosql(name, wrapped_ty);
    let as_expr_impl = gen_asexpressions(name, wrapped_ty);

    // raw deserialization
    let from_sql_impl = gen_from_sql(name, wrapped_ty);

    // querying
    let queryable_impl = gen_queryable(name, wrapped_ty);

    // since our query doesn't take varargs it's fine for the DB to cache it
    let query_id_impl = gen_query_id(name);

    wrap_impls_in_const(&quote! {
        #to_sql_impl
        #as_expr_impl

        #from_sql_impl

        #queryable_impl

        #query_id_impl
    })
}

fn gen_tosql(name: &syn::Ident, wrapped_ty: &syn::Type) -> TokenStream {
    quote! {
        impl<ST, DB> diesel::serialize::ToSql<ST, DB> for #name
        where
            #wrapped_ty: diesel::serialize::ToSql<ST, DB>,
            DB: diesel::backend::Backend,
            DB: diesel::sql_types::HasSqlType<ST>,
        {
            fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, DB>) -> diesel::serialize::Result
            {
                self.0.to_sql(out)
            }
        }
    }
}

fn gen_asexpressions(name: &syn::Ident, wrapped_ty: &syn::Type) -> TokenStream {
    quote! {

        impl<ST> diesel::expression::AsExpression<ST> for #name
        where
            diesel::internal::derives::as_expression::Bound<ST, #wrapped_ty>:
                diesel::expression::Expression<SqlType=ST>,
            ST: diesel::sql_types::SingleValue,
        {
            type Expression = diesel::internal::derives::as_expression::Bound<ST, Self>;

            fn as_expression(self) -> Self::Expression {
                diesel::internal::derives::as_expression::Bound::new(self)
            }
        }

        impl<'expr, ST> diesel::expression::AsExpression<ST> for &'expr #name
        where
            diesel::internal::derives::as_expression::Bound<ST, #wrapped_ty>:
                diesel::expression::Expression<SqlType=ST>,
            ST: diesel::sql_types::SingleValue,
        {
            type Expression = diesel::internal::derives::as_expression::Bound<ST, Self>;

            fn as_expression(self) -> Self::Expression {
                diesel::internal::derives::as_expression::Bound::new(self)
            }
        }

        impl<'expr2, 'expr, ST> diesel::expression::AsExpression<ST> for &'expr2 &'expr #name
        where
            diesel::internal::derives::as_expression::Bound<ST, #wrapped_ty>:
                diesel::expression::Expression<SqlType=ST>,
            ST: diesel::sql_types::SingleValue,
        {
            type Expression = diesel::internal::derives::as_expression::Bound<ST, Self>;

            fn as_expression(self) -> Self::Expression {
                diesel::internal::derives::as_expression::Bound::new(self)
            }
        }
    }
}

fn gen_from_sql(name: &syn::Ident, wrapped_ty: &syn::Type) -> TokenStream {
    quote! {
        impl<ST, DB> diesel::deserialize::FromSql<ST, DB> for #name
        where
            #wrapped_ty: diesel::deserialize::FromSql<ST, DB>,
            DB: diesel::backend::Backend,
            DB: diesel::sql_types::HasSqlType<ST>,
        {
            fn from_sql(raw: DB::RawValue<'_>)
            -> ::std::result::Result<Self, Box<::std::error::Error + Send + Sync>>
            {
                diesel::deserialize::FromSql::<ST, DB>::from_sql(raw)
                    .map(#name)
            }
        }
    }
}

fn gen_queryable(name: &syn::Ident, wrapped_ty: &syn::Type) -> TokenStream {
    quote! {
        impl<ST, DB> diesel::deserialize::Queryable<ST, DB> for #name
        where
            #wrapped_ty: diesel::deserialize::FromStaticSqlRow<ST, DB>,
            DB: diesel::backend::Backend,
            DB: diesel::sql_types::HasSqlType<ST>,
        {
            type Row = #wrapped_ty;

            fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
                Ok(#name(row))
            }
        }
    }
}

fn gen_query_id(name: &syn::Ident) -> TokenStream {
    quote! {
        impl diesel::query_builder::QueryId for #name {
            type QueryId = Self;
        }
    }
}

/// This guarantees that items we generate don't pollute the module scope
fn wrap_impls_in_const(item: &TokenStream) -> TokenStream {
    let dummy_const = syn::Ident::new("_", Span::call_site());
    quote! {
        const #dummy_const: () = {
            #item
        };
    }
}
