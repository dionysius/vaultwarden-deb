//! Low-level crate implementing various RFCs for encoding emails.
//! Used internally by [lettre].
//!
//! [lettre]: https://crates.io/crates/lettre

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(
    unreachable_pub,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    rust_2018_idioms,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::clone_on_ref_ptr,
    clippy::verbose_file_reads,
    clippy::unnecessary_self_imports,
    clippy::string_to_string,
    clippy::mem_forget,
    clippy::cast_lossless,
    clippy::inefficient_to_string,
    clippy::inline_always,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::unnecessary_join,
    clippy::wildcard_imports,
    clippy::str_to_string,
    clippy::empty_structs_with_brackets,
    clippy::zero_sized_map_values,
    clippy::manual_let_else,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_wraps,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::redundant_closure_for_method_calls,
// Rust 1.86: clippy::unnecessary_semicolon,
)]

#[cfg(test)]
extern crate alloc;

pub mod body;
pub mod headers;
