//!
//! `num-order` implements numerically consistent [Eq][core::cmp::Eq], [Ord][core::cmp::Ord] and
//! [Hash][core::hash::Hash] for various `num` types.
//!
//! ```rust
//! use std::cmp::Ordering;
//! use std::hash::Hasher;
//! use std::collections::hash_map::DefaultHasher;
//! use num_order::NumOrd;
//!
//! assert!(NumOrd::num_eq(&3u64, &3.0f32));
//! assert!(NumOrd::num_lt(&-4.7f64, &-4i8));
//! assert!(!NumOrd::num_ge(&-3i8, &1u16));
//!
//! // 40_000_000 can be exactly represented in f32, 40_000_001 cannot
//! // 40_000_001 becames 40_000_000.0 in f32
//! assert_eq!(NumOrd::num_cmp(&40_000_000f32, &40_000_000u32), Ordering::Equal);
//! assert_ne!(NumOrd::num_cmp(&40_000_001f32, &40_000_001u32), Ordering::Equal);
//! assert_eq!(NumOrd::num_partial_cmp(&f32::NAN, &40_000_002u32), None);
//!
//! use num_order::NumHash;
//! // same hash values are guaranteed for equal numbers
//! let mut hasher1 = DefaultHasher::new();
//! 3u64.num_hash(&mut hasher1);
//! let mut hasher2 = DefaultHasher::new();
//! 3.0f32.num_hash(&mut hasher2);
//! assert_eq!(hasher1.finish(), hasher2.finish())
//! ```
//!
//! This crate can serve applications where [float-ord](https://crates.io/crates/float-ord),
//! [num-cmp](https://crates.io/crates/num-cmp), [num-ord](https://crates.io/crates/num-ord) are used.
//! Meanwhile it also supports hashing and more numeric types (`num-bigint`, etc.).
//!
//! # Optional Features
//! - `std`: enable std library
//! - `num-bigint`: Support comparing against and hashing `num-bigint::{BigInt, BigUint}`
//! - `num-rational`: Support comparing against and hashing `num-rational::Ratio<I>`, where `I` can be
//!     `i8`, `i16`, `i32`, `i64`, `i128` and `isize`. `Ratio<BigInt>` is supported when both `num-bigint`
//!     and `num-rational` is enabled
//! - `num-complex`: Support comparing against and hashing `num-complex::{Complex32, Complex64}`
//!

#![no_std]
#[cfg(any(feature = "std", test))]
extern crate std;

use core::cmp::Ordering;
use core::hash::Hasher;

/// Consistent comparison among different numeric types.
pub trait NumOrd<Other> {
    /// [PartialOrd::partial_cmp] on different numeric types
    fn num_partial_cmp(&self, other: &Other) -> Option<Ordering>;

    #[inline]
    /// [PartialEq::eq] on different numeric types
    fn num_eq(&self, other: &Other) -> bool {
        matches!(self.num_partial_cmp(other), Some(Ordering::Equal))
    }
    #[inline]
    /// [PartialEq::ne] on different numeric types
    fn num_ne(&self, other: &Other) -> bool {
        !self.num_eq(other)
    }
    #[inline]
    /// [PartialOrd::lt] on different numeric types
    fn num_lt(&self, other: &Other) -> bool {
        matches!(self.num_partial_cmp(other), Some(Ordering::Less))
    }
    #[inline]
    /// [PartialOrd::le] on different numeric types
    fn num_le(&self, other: &Other) -> bool {
        matches!(
            self.num_partial_cmp(other),
            Some(Ordering::Equal) | Some(Ordering::Less)
        )
    }
    #[inline]
    /// [PartialOrd::gt] on different numeric types
    fn num_gt(&self, other: &Other) -> bool {
        matches!(self.num_partial_cmp(other), Some(Ordering::Greater))
    }
    #[inline]
    /// [PartialOrd::ge] on different numeric types
    fn num_ge(&self, other: &Other) -> bool {
        matches!(
            self.num_partial_cmp(other),
            Some(Ordering::Equal) | Some(Ordering::Greater)
        )
    }
    #[inline]
    /// [Ord::cmp] on different numeric types. It panics if either of the numeric values contains NaN.
    fn num_cmp(&self, other: &Other) -> Ordering {
        self.num_partial_cmp(other).unwrap()
    }
}

/// Consistent hash implementation among different numeric types.
///
/// It's ensured that if `a.num_eq(b)`, then `a` and `b` will result in the same hash. Although the other direction is
/// not ensured because it's infeasible, the hash function is still designed to be as sparse as possible.
pub trait NumHash {
    /// Consistent [Hash::hash][core::hash::Hash::hash] on different numeric types.
    ///
    /// This function will ensures if `a.num_eq(b)`, then `a.num_hash()` and `b.num_hash()` manipulate the state in the same way.
    fn num_hash<H: Hasher>(&self, state: &mut H);
}

mod hash;
mod ord;
#[cfg(test)]
mod tests;

// TODO: support num-irrational::{QuadraticSurd, QuadraticInt} when their API is stablized
