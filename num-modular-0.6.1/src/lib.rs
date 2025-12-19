//! This crate provides efficient Modular arithmetic operations for various integer types,
//! including primitive integers and `num-bigint`. The latter option is enabled optionally.
//!
//! To achieve fast modular arithmetics, convert integers to any [ModularInteger] implementation
//! use static `new()` or associated [ModularInteger::convert()] functions. Some builtin implementations
//! of [ModularInteger] includes [MontgomeryInt] and [FixedMersenneInt].
//!
//! Example code:
//! ```rust
//! use num_modular::{ModularCoreOps, ModularInteger, MontgomeryInt};
//!
//! // directly using methods in ModularCoreOps
//! let (x, y, m) = (12u8, 13u8, 5u8);
//! assert_eq!(x.mulm(y, &m), x * y % m);
//!
//! // convert integers into ModularInteger
//! let mx = MontgomeryInt::new(x, &m);
//! let my = mx.convert(y); // faster than static MontgomeryInt::new(y, m)
//! assert_eq!((mx * my).residue(), x * y % m);
//! ```
//!
//! # Comparison of fast division / modular arithmetics
//! Several fast division / modulo tricks are provided in these crate, the difference of them are listed below:
//! - [PreModInv]: pre-compute modular inverse of the divisor, only applicable to exact division
//! - Barret (to be implemented): pre-compute (rational approximation of) the reciprocal of the divisor,
//!     applicable to fast division and modulo
//! - [Montgomery]: Convert the dividend into a special form by shifting and pre-compute a modular inverse,
//!     only applicable to fast modulo, but faster than Barret reduction
//! - [FixedMersenne]: Specialization of modulo in form `2^P-K` under 2^127.
//!

// XXX: Other fast modular arithmetic tricks
// REF: https://github.com/lemire/fastmod & https://arxiv.org/pdf/1902.01961.pdf
// REF: https://eprint.iacr.org/2014/040.pdf
// REF: https://github.com/ridiculousfish/libdivide/
// REF: Faster Interleaved Modular Multiplication Based on Barrett and Montgomery Reduction Methods (work for modulus in certain form)

#![no_std]
#[cfg(any(feature = "std", test))]
extern crate std;

use core::ops::{Add, Mul, Neg, Sub};

/// Core modular arithmetic operations.
///
/// Note that all functions will panic if the modulus is zero.
pub trait ModularCoreOps<Rhs = Self, Modulus = Self> {
    type Output;

    /// Return (self + rhs) % m
    fn addm(self, rhs: Rhs, m: Modulus) -> Self::Output;

    /// Return (self - rhs) % m
    fn subm(self, rhs: Rhs, m: Modulus) -> Self::Output;

    /// Return (self * rhs) % m
    fn mulm(self, rhs: Rhs, m: Modulus) -> Self::Output;
}

/// Core unary modular arithmetics
///
/// Note that all functions will panic if the modulus is zero.
pub trait ModularUnaryOps<Modulus = Self> {
    type Output;

    /// Return (-self) % m and make sure the result is normalized in range [0,m)
    fn negm(self, m: Modulus) -> Self::Output;

    /// Calculate modular inverse (x such that self*x = 1 mod m).
    ///
    /// This operation is only available for integer that is coprime to `m`. If not,
    /// the result will be [None].
    fn invm(self, m: Modulus) -> Option<Self::Output>;

    /// Calculate modular double ( x+x mod m)
    fn dblm(self, m: Modulus) -> Self::Output;

    /// Calculate modular square ( x*x mod m )
    fn sqm(self, m: Modulus) -> Self::Output;

    // TODO: Modular sqrt aka Quadratic residue, follow the behavior of FLINT `n_sqrtmod`
    // fn sqrtm(self, m: Modulus) -> Option<Self::Output>;
    // REF: https://stackoverflow.com/questions/6752374/cube-root-modulo-p-how-do-i-do-this
}

/// Modular power functions
pub trait ModularPow<Exp = Self, Modulus = Self> {
    type Output;

    /// Return (self ^ exp) % m
    fn powm(self, exp: Exp, m: Modulus) -> Self::Output;
}

/// Math symbols related to modular arithmetics
pub trait ModularSymbols<Modulus = Self> {
    /// Calculate Legendre Symbol (a|n), where a is `self`.
    ///
    /// Note that this function doesn't perform a full primality check, since
    /// is costly. So if n is not a prime, the result can be not reasonable.
    ///
    /// # Panics
    /// Only if n is not prime
    #[inline]
    fn legendre(&self, n: Modulus) -> i8 {
        self.checked_legendre(n).expect("n shoud be a prime")
    }

    /// Calculate Legendre Symbol (a|n), where a is `self`. Returns [None] only if n is
    /// not a prime.
    ///
    /// Note that this function doesn't perform a full primality check, since
    /// is costly. So if n is not a prime, the result can be not reasonable.
    ///
    /// # Panics
    /// Only if n is not prime
    fn checked_legendre(&self, n: Modulus) -> Option<i8>;

    /// Calculate Jacobi Symbol (a|n), where a is `self`
    ///
    /// # Panics
    /// if n is negative or even
    #[inline]
    fn jacobi(&self, n: Modulus) -> i8 {
        self.checked_jacobi(n)
            .expect("the Jacobi symbol is only defined for non-negative odd integers")
    }

    /// Calculate Jacobi Symbol (a|n), where a is `self`. Returns [None] if n is negative or even.
    fn checked_jacobi(&self, n: Modulus) -> Option<i8>;

    /// Calculate Kronecker Symbol (a|n), where a is `self`
    fn kronecker(&self, n: Modulus) -> i8;
}

// TODO: Discrete log aka index, follow the behavior of FLINT `n_discrete_log_bsgs`
// REF: https://github.com/vks/discrete-log
// fn logm(self, base: Modulus, m: Modulus);

/// Collection of common modular arithmetic operations
pub trait ModularOps<Rhs = Self, Modulus = Self, Output = Self>:
    ModularCoreOps<Rhs, Modulus, Output = Output>
    + ModularUnaryOps<Modulus, Output = Output>
    + ModularPow<Rhs, Modulus, Output = Output>
    + ModularSymbols<Modulus>
{
}
impl<T, Rhs, Modulus> ModularOps<Rhs, Modulus> for T where
    T: ModularCoreOps<Rhs, Modulus, Output = T>
        + ModularUnaryOps<Modulus, Output = T>
        + ModularPow<Rhs, Modulus, Output = T>
        + ModularSymbols<Modulus>
{
}

/// Collection of operations similar to [ModularOps], but takes operands with references
pub trait ModularRefOps: for<'r> ModularOps<&'r Self, &'r Self> + Sized {}
impl<T> ModularRefOps for T where T: for<'r> ModularOps<&'r T, &'r T> {}

/// Provides a utility function to convert signed integers into unsigned modular form
pub trait ModularAbs<Modulus> {
    /// Return self % m, but accepting signed integers
    fn absm(self, m: &Modulus) -> Modulus;
}

/// Represents an number defined in a modulo ring ℤ/nℤ
///
/// The operators should panic if the modulus of two number
/// are not the same.
pub trait ModularInteger:
    Sized
    + PartialEq
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Neg<Output = Self>
    + Mul<Self, Output = Self>
{
    /// The underlying representation type of the integer
    type Base;

    /// Return the modulus of the ring
    fn modulus(&self) -> Self::Base;

    /// Return the normalized residue of this integer in the ring
    fn residue(&self) -> Self::Base;

    /// Check if the integer is zero
    fn is_zero(&self) -> bool;

    /// Convert an normal integer into the same ring.
    ///
    /// This method should be perferred over the static
    /// constructor to prevent unnecessary overhead of pre-computation.
    fn convert(&self, n: Self::Base) -> Self;

    /// Calculate the value of self + self
    fn double(self) -> Self;

    /// Calculate the value of self * self
    fn square(self) -> Self;
}

// XXX: implement ModularInteger for ff::PrimeField?
// TODO: implement invm_range (Modular inverse in certain range) and crt (Chinese Remainder Theorem), REF: bubblemath crate

/// Utility function for exact division, with precomputed helper values
///
/// # Available Pre-computation types:
/// - `()`: No pre-computation, the implementation relies on native integer division
/// - [PreModInv]: With Pre-computed modular inverse
pub trait DivExact<Rhs, Precompute>: Sized {
    type Output;

    /// Check if d divides self with the help of the precomputation. If d divides self,
    /// then the quotient is returned.
    fn div_exact(self, d: Rhs, pre: &Precompute) -> Option<Self::Output>;
}

/// A modular reducer that can ensure that the operations on integers are all performed
/// in a modular ring.
///
/// Essential information for performing the modulo operation will be stored in the reducer.
pub trait Reducer<T> {
    /// Create a reducer for a modulus m
    fn new(m: &T) -> Self;

    /// Transform a normal integer into reduced form
    fn transform(&self, target: T) -> T;

    /// Check whether target is a valid reduced form
    fn check(&self, target: &T) -> bool;

    /// Get the modulus in original integer type
    fn modulus(&self) -> T;

    /// Transform a reduced form back to normal integer
    fn residue(&self, target: T) -> T;

    /// Test if the residue() == 0
    fn is_zero(&self, target: &T) -> bool;

    /// Calculate (lhs + rhs) mod m in reduced form
    fn add(&self, lhs: &T, rhs: &T) -> T;

    #[inline]
    fn add_in_place(&self, lhs: &mut T, rhs: &T) {
        *lhs = self.add(lhs, rhs)
    }

    /// Calculate 2*target mod m
    fn dbl(&self, target: T) -> T;

    /// Calculate (lhs - rhs) mod m in reduced form
    fn sub(&self, lhs: &T, rhs: &T) -> T;

    #[inline]
    fn sub_in_place(&self, lhs: &mut T, rhs: &T) {
        *lhs = self.sub(lhs, rhs);
    }

    /// Calculate -monty mod m in reduced form
    fn neg(&self, target: T) -> T;

    /// Calculate (lhs * rhs) mod m in reduced form
    fn mul(&self, lhs: &T, rhs: &T) -> T;

    #[inline]
    fn mul_in_place(&self, lhs: &mut T, rhs: &T) {
        *lhs = self.mul(lhs, rhs);
    }

    /// Calculate target^-1 mod m in reduced form,
    /// it may return None when there is no modular inverse.
    fn inv(&self, target: T) -> Option<T>;

    /// Calculate target^2 mod m in reduced form
    fn sqr(&self, target: T) -> T;

    /// Calculate base ^ exp mod m in reduced form
    fn pow(&self, base: T, exp: &T) -> T;
}

mod barret;
mod double;
mod mersenne;
mod monty;
mod preinv;
mod prim;
mod reduced;
mod word;

pub use barret::{
    Normalized2by1Divisor, Normalized3by2Divisor, PreMulInv1by1, PreMulInv2by1, PreMulInv3by2,
};
pub use double::{udouble, umax};
pub use mersenne::FixedMersenne;
pub use monty::Montgomery;
pub use preinv::PreModInv;
pub use reduced::{ReducedInt, Vanilla, VanillaInt};

/// An integer in modulo ring based on [Montgomery form](https://en.wikipedia.org/wiki/Montgomery_modular_multiplication#Montgomery_form)
pub type MontgomeryInt<T> = ReducedInt<T, Montgomery<T>>;

/// An integer in modulo ring with a fixed (pseudo) Mersenne number as modulus
pub type FixedMersenneInt<const P: u8, const K: umax> = ReducedInt<umax, FixedMersenne<P, K>>;

// pub type BarretInt<T> = ReducedInt<T, BarretInt<T>>;

#[cfg(feature = "num-bigint")]
mod bigint;
