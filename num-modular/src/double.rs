//! This module implements a double width integer type based on the largest built-in integer (u128)
//! Part of the optimization comes from `ethnum` and `zkp-u256` crates.

use core::ops::*;

/// Alias of the builtin integer type with max width (currently [u128])
#[allow(non_camel_case_types)]
pub type umax = u128;

const HALF_BITS: u32 = umax::BITS / 2;

// Split umax into hi and lo parts. Tt's critical to use inline here
#[inline(always)]
const fn split(v: umax) -> (umax, umax) {
    (v >> HALF_BITS, v & (umax::MAX >> HALF_BITS))
}

#[inline(always)]
const fn div_rem(n: umax, d: umax) -> (umax, umax) {
    (n / d, n % d)
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A double width integer type based on the largest built-in integer type [umax] (currently [u128]), and
/// to support double-width operations on it is the only goal for this type.
///
/// Although it can be regarded as u256, it's not as feature-rich as in other crates
/// since it's only designed to support this crate and few other crates (will be noted in comments).
pub struct udouble {
    /// Most significant part
    pub hi: umax,
    /// Least significant part
    pub lo: umax,
}

impl udouble {
    pub const MAX: Self = Self {
        lo: umax::MAX,
        hi: umax::MAX,
    };

    //> (used in u128::addm)
    #[inline]
    pub const fn widening_add(lhs: umax, rhs: umax) -> Self {
        let (sum, carry) = lhs.overflowing_add(rhs);
        udouble {
            hi: carry as umax,
            lo: sum,
        }
    }

    /// Calculate multiplication of two [umax] integers with result represented in double width integer
    // equivalent to umul_ppmm, can be implemented efficiently with carrying_mul and widening_mul implemented (rust#85532)
    //> (used in u128::mulm, MersenneInt, Montgomery::<u128>::{reduce, mul}, num-order::NumHash)
    #[inline]
    pub const fn widening_mul(lhs: umax, rhs: umax) -> Self {
        let ((x1, x0), (y1, y0)) = (split(lhs), split(rhs));

        let z2 = x1 * y1;
        let (c0, z0) = split(x0 * y0); // c0 <= umax::MAX - 1
        let (c1, z1) = split(x1 * y0 + c0);
        let z2 = z2 + c1;
        let (c1, z1) = split(x0 * y1 + z1);
        Self {
            hi: z2 + c1,
            lo: z0 | z1 << HALF_BITS,
        }
    }

    /// Optimized squaring function for [umax] integers
    //> (used in Montgomery::<u128>::{square})
    #[inline]
    pub const fn widening_square(x: umax) -> Self {
        // the algorithm here is basically the same as widening_mul
        let (x1, x0) = split(x);

        let z2 = x1 * x1;
        let m = x1 * x0;
        let (c0, z0) = split(x0 * x0);
        let (c1, z1) = split(m + c0);
        let z2 = z2 + c1;
        let (c1, z1) = split(m + z1);
        Self {
            hi: z2 + c1,
            lo: z0 | z1 << HALF_BITS,
        }
    }

    //> (used in Montgomery::<u128>::reduce)
    #[inline]
    pub const fn overflowing_add(&self, rhs: Self) -> (Self, bool) {
        let (lo, carry) = self.lo.overflowing_add(rhs.lo);
        let (hi, of1) = self.hi.overflowing_add(rhs.hi);
        let (hi, of2) = hi.overflowing_add(carry as umax);
        (Self { lo, hi }, of1 || of2)
    }

    // double by double multiplication, listed here in case of future use
    #[allow(dead_code)]
    fn overflowing_mul(&self, rhs: Self) -> (Self, bool) {
        let c2 = self.hi != 0 && rhs.hi != 0;
        let Self { lo: z0, hi: c0 } = Self::widening_mul(self.lo, rhs.lo);
        let (z1x, c1x) = umax::overflowing_mul(self.lo, rhs.hi);
        let (z1y, c1y) = umax::overflowing_mul(self.hi, rhs.lo);
        let (z1z, c1z) = umax::overflowing_add(z1x, z1y);
        let (z1, c1) = z1z.overflowing_add(c0);
        (Self { hi: z1, lo: z0 }, c1x | c1y | c1z | c1 | c2)
    }

    /// Multiplication of double width and single width
    //> (used in num-order:NumHash)
    #[inline]
    pub const fn overflowing_mul1(&self, rhs: umax) -> (Self, bool) {
        let Self { lo: z0, hi: c0 } = Self::widening_mul(self.lo, rhs);
        let (z1, c1) = self.hi.overflowing_mul(rhs);
        let (z1, cs1) = z1.overflowing_add(c0);
        (Self { hi: z1, lo: z0 }, c1 | cs1)
    }

    /// Multiplication of double width and single width
    //> (used in Self::mul::<umax>)
    #[inline]
    pub fn checked_mul1(&self, rhs: umax) -> Option<Self> {
        let Self { lo: z0, hi: c0 } = Self::widening_mul(self.lo, rhs);
        let z1 = self.hi.checked_mul(rhs)?.checked_add(c0)?;
        Some(Self { hi: z1, lo: z0 })
    }

    //> (used in num-order::NumHash)
    #[inline]
    pub fn checked_shl(self, rhs: u32) -> Option<Self> {
        if rhs < umax::BITS * 2 {
            Some(self << rhs)
        } else {
            None
        }
    }

    //> (not used yet)
    #[inline]
    pub fn checked_shr(self, rhs: u32) -> Option<Self> {
        if rhs < umax::BITS * 2 {
            Some(self >> rhs)
        } else {
            None
        }
    }
}

impl From<umax> for udouble {
    #[inline]
    fn from(v: umax) -> Self {
        Self { lo: v, hi: 0 }
    }
}

impl Add for udouble {
    type Output = udouble;

    // equivalent to add_ssaaaa
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let (lo, carry) = self.lo.overflowing_add(rhs.lo);
        let hi = self.hi + rhs.hi + carry as umax;
        Self { lo, hi }
    }
}
//> (used in Self::div_rem)
impl Add<umax> for udouble {
    type Output = udouble;
    #[inline]
    fn add(self, rhs: umax) -> Self::Output {
        let (lo, carry) = self.lo.overflowing_add(rhs);
        let hi = if carry { self.hi + 1 } else { self.hi };
        Self { lo, hi }
    }
}
impl AddAssign for udouble {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        let (lo, carry) = self.lo.overflowing_add(rhs.lo);
        self.lo = lo;
        self.hi += rhs.hi + carry as umax;
    }
}
impl AddAssign<umax> for udouble {
    #[inline]
    fn add_assign(&mut self, rhs: umax) {
        let (lo, carry) = self.lo.overflowing_add(rhs);
        self.lo = lo;
        if carry {
            self.hi += 1
        }
    }
}

//> (used in test of Add)
impl Sub for udouble {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let carry = self.lo < rhs.lo;
        let lo = self.lo.wrapping_sub(rhs.lo);
        let hi = self.hi - rhs.hi - carry as umax;
        Self { lo, hi }
    }
}
impl Sub<umax> for udouble {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: umax) -> Self::Output {
        let carry = self.lo < rhs;
        let lo = self.lo.wrapping_sub(rhs);
        let hi = if carry { self.hi - 1 } else { self.hi };
        Self { lo, hi }
    }
}
//> (used in test of AddAssign)
impl SubAssign for udouble {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        let carry = self.lo < rhs.lo;
        self.lo = self.lo.wrapping_sub(rhs.lo);
        self.hi -= rhs.hi + carry as umax;
    }
}
impl SubAssign<umax> for udouble {
    #[inline]
    fn sub_assign(&mut self, rhs: umax) {
        let carry = self.lo < rhs;
        self.lo = self.lo.wrapping_sub(rhs);
        if carry {
            self.hi -= 1;
        }
    }
}

macro_rules! impl_sh_ops {
    ($t:ty) => {
        //> (used in Self::checked_shl)
        impl Shl<$t> for udouble {
            type Output = Self;
            #[inline]
            fn shl(self, rhs: $t) -> Self::Output {
                match rhs {
                    0 => self, // avoid shifting by full bits, which is UB
                    s if s >= umax::BITS as $t => Self {
                        hi: self.lo << (s - umax::BITS as $t),
                        lo: 0,
                    },
                    s => Self {
                        lo: self.lo << s,
                        hi: (self.hi << s) | (self.lo >> (umax::BITS as $t - s)),
                    },
                }
            }
        }
        //> (not used yet)
        impl ShlAssign<$t> for udouble {
            #[inline]
            fn shl_assign(&mut self, rhs: $t) {
                match rhs {
                    0 => {}
                    s if s >= umax::BITS as $t => {
                        self.hi = self.lo << (s - umax::BITS as $t);
                        self.lo = 0;
                    }
                    s => {
                        self.hi <<= s;
                        self.hi |= self.lo >> (umax::BITS as $t - s);
                        self.lo <<= s;
                    }
                }
            }
        }
        //> (used in Self::checked_shr)
        impl Shr<$t> for udouble {
            type Output = Self;
            #[inline]
            fn shr(self, rhs: $t) -> Self::Output {
                match rhs {
                    0 => self,
                    s if s >= umax::BITS as $t => Self {
                        lo: self.hi >> (rhs - umax::BITS as $t),
                        hi: 0,
                    },
                    s => Self {
                        hi: self.hi >> s,
                        lo: (self.lo >> s) | (self.hi << (umax::BITS as $t - s)),
                    },
                }
            }
        }
        //> (not used yet)
        impl ShrAssign<$t> for udouble {
            #[inline]
            fn shr_assign(&mut self, rhs: $t) {
                match rhs {
                    0 => {}
                    s if s >= umax::BITS as $t => {
                        self.lo = self.hi >> (rhs - umax::BITS as $t);
                        self.hi = 0;
                    }
                    s => {
                        self.lo >>= s;
                        self.lo |= self.hi << (umax::BITS as $t - s);
                        self.hi >>= s;
                    }
                }
            }
        }
    };
}

// only implement most useful ones, so that we don't need to optimize so many variants
impl_sh_ops!(u8);
impl_sh_ops!(u16);
impl_sh_ops!(u32);

//> (not used yet)
impl BitAnd for udouble {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo & rhs.lo,
            hi: self.hi & rhs.hi,
        }
    }
}
//> (not used yet)
impl BitAndAssign for udouble {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.lo &= rhs.lo;
        self.hi &= rhs.hi;
    }
}
//> (not used yet)
impl BitOr for udouble {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo | rhs.lo,
            hi: self.hi | rhs.hi,
        }
    }
}
//> (not used yet)
impl BitOrAssign for udouble {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.lo |= rhs.lo;
        self.hi |= rhs.hi;
    }
}
//> (not used yet)
impl BitXor for udouble {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo ^ rhs.lo,
            hi: self.hi ^ rhs.hi,
        }
    }
}
//> (not used yet)
impl BitXorAssign for udouble {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.lo ^= rhs.lo;
        self.hi ^= rhs.hi;
    }
}
//> (not used yet)
impl Not for udouble {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self {
            lo: !self.lo,
            hi: !self.hi,
        }
    }
}

impl udouble {
    //> (used in Self::div_rem)
    #[inline]
    pub const fn leading_zeros(self) -> u32 {
        if self.hi == 0 {
            self.lo.leading_zeros() + umax::BITS
        } else {
            self.hi.leading_zeros()
        }
    }

    // double by double division (long division), it's not the most efficient algorithm.
    // listed here in case of future use
    #[allow(dead_code)]
    fn div_rem_2by2(self, other: Self) -> (Self, Self) {
        let mut n = self; // numerator
        let mut d = other; // denominator
        let mut q = Self { lo: 0, hi: 0 }; // quotient

        let nbits = (2 * umax::BITS - n.leading_zeros()) as u16; // assuming umax = u128
        let dbits = (2 * umax::BITS - d.leading_zeros()) as u16;
        assert!(dbits != 0, "division by zero");

        // Early return in case we are dividing by a larger number than us
        if nbits < dbits {
            return (q, n);
        }

        // Bitwise long division
        let mut shift = nbits - dbits;
        d <<= shift;
        loop {
            if n >= d {
                q += 1;
                n -= d;
            }
            if shift == 0 {
                break;
            }

            d >>= 1u8;
            q <<= 1u8;
            shift -= 1;
        }
        (q, n)
    }

    // double by single to single division.
    // equivalent to `udiv_qrnnd` in C or `divq` in assembly.
    //> (used in Self::{div, rem}::<umax>)
    fn div_rem_2by1(self, other: umax) -> (umax, umax) {
        // the following algorithm comes from `ethnum` crate
        const B: umax = 1 << HALF_BITS; // number base (64 bits)

        // Normalize the divisor.
        let s = other.leading_zeros();
        let (n, d) = (self << s, other << s); // numerator, denominator
        let (d1, d0) = split(d);
        let (n1, n0) = split(n.lo); // split lower part of dividend

        // Compute the first quotient digit q1.
        let (mut q1, mut rhat) = div_rem(n.hi, d1);

        // q1 has at most error 2. No more than 2 iterations.
        while q1 >= B || q1 * d0 > B * rhat + n1 {
            q1 -= 1;
            rhat += d1;
            if rhat >= B {
                break;
            }
        }

        let r21 =
            n.hi.wrapping_mul(B)
                .wrapping_add(n1)
                .wrapping_sub(q1.wrapping_mul(d));

        // Compute the second quotient digit q0.
        let (mut q0, mut rhat) = div_rem(r21, d1);

        // q0 has at most error 2. No more than 2 iterations.
        while q0 >= B || q0 * d0 > B * rhat + n0 {
            q0 -= 1;
            rhat += d1;
            if rhat >= B {
                break;
            }
        }

        let r = (r21
            .wrapping_mul(B)
            .wrapping_add(n0)
            .wrapping_sub(q0.wrapping_mul(d)))
            >> s;
        let q = q1 * B + q0;
        (q, r)
    }
}

impl Mul<umax> for udouble {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: umax) -> Self::Output {
        self.checked_mul1(rhs).expect("multiplication overflow!")
    }
}

impl Div<umax> for udouble {
    type Output = Self;
    #[inline]
    fn div(self, rhs: umax) -> Self::Output {
        // self.div_rem(rhs.into()).0
        if self.hi < rhs {
            // The result fits in 128 bits.
            Self {
                lo: self.div_rem_2by1(rhs).0,
                hi: 0,
            }
        } else {
            let (q, r) = div_rem(self.hi, rhs);
            Self {
                lo: Self { lo: self.lo, hi: r }.div_rem_2by1(rhs).0,
                hi: q,
            }
        }
    }
}

//> (used in Montgomery::<u128>::transform)
impl Rem<umax> for udouble {
    type Output = umax;
    #[inline]
    fn rem(self, rhs: umax) -> Self::Output {
        if self.hi < rhs {
            // The result fits in 128 bits.
            self.div_rem_2by1(rhs).1
        } else {
            Self {
                lo: self.lo,
                hi: self.hi % rhs,
            }
            .div_rem_2by1(rhs)
            .1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    #[test]
    fn test_construction() {
        // from widening operators
        assert_eq!(udouble { hi: 0, lo: 2 }, udouble::widening_add(1, 1));
        assert_eq!(
            udouble {
                hi: 1,
                lo: umax::MAX - 1
            },
            udouble::widening_add(umax::MAX, umax::MAX)
        );

        assert_eq!(udouble { hi: 0, lo: 1 }, udouble::widening_mul(1, 1));
        assert_eq!(udouble { hi: 0, lo: 1 }, udouble::widening_square(1));
        assert_eq!(
            udouble { hi: 1 << 32, lo: 0 },
            udouble::widening_mul(1 << 80, 1 << 80)
        );
        assert_eq!(
            udouble { hi: 1 << 32, lo: 0 },
            udouble::widening_square(1 << 80)
        );
        assert_eq!(
            udouble {
                hi: 1 << 32,
                lo: 2 << 120 | 1 << 80
            },
            udouble::widening_mul(1 << 80 | 1 << 40, 1 << 80 | 1 << 40)
        );
        assert_eq!(
            udouble {
                hi: 1 << 32,
                lo: 2 << 120 | 1 << 80
            },
            udouble::widening_square(1 << 80 | 1 << 40)
        );
        assert_eq!(
            udouble {
                hi: umax::MAX - 1,
                lo: 1
            },
            udouble::widening_mul(umax::MAX, umax::MAX)
        );
        assert_eq!(
            udouble {
                hi: umax::MAX - 1,
                lo: 1
            },
            udouble::widening_square(umax::MAX)
        );
    }

    #[test]
    fn test_ops() {
        const ONE: udouble = udouble { hi: 0, lo: 1 };
        const TWO: udouble = udouble { hi: 0, lo: 2 };
        const MAX: udouble = udouble {
            hi: 0,
            lo: umax::MAX,
        };
        const ONEZERO: udouble = udouble { hi: 1, lo: 0 };
        const ONEMAX: udouble = udouble {
            hi: 1,
            lo: umax::MAX,
        };
        const TWOZERO: udouble = udouble { hi: 2, lo: 0 };

        assert_eq!(ONE + MAX, ONEZERO);
        assert_eq!(ONE + ONEMAX, TWOZERO);
        assert_eq!(ONEZERO - ONE, MAX);
        assert_eq!(ONEZERO - MAX, ONE);
        assert_eq!(TWOZERO - ONE, ONEMAX);
        assert_eq!(TWOZERO - ONEMAX, ONE);

        assert_eq!(ONE << umax::BITS, ONEZERO);
        assert_eq!((MAX << 1u8) + 1, ONEMAX);
        assert_eq!(
            ONE << 200u8,
            udouble {
                lo: 0,
                hi: 1 << (200 - umax::BITS)
            }
        );
        assert_eq!(ONEZERO >> umax::BITS, ONE);
        assert_eq!(ONEMAX >> 1u8, MAX);

        assert_eq!(ONE * MAX.lo, MAX);
        assert_eq!(ONEMAX * ONE.lo, ONEMAX);
        assert_eq!(ONEMAX * TWO.lo, ONEMAX + ONEMAX);
        assert_eq!(MAX / ONE.lo, MAX);
        assert_eq!(MAX / MAX.lo, ONE);
        assert_eq!(ONE / MAX.lo, udouble { lo: 0, hi: 0 });
        assert_eq!(ONEMAX / ONE.lo, ONEMAX);
        assert_eq!(ONEMAX / MAX.lo, TWO);
        assert_eq!(ONEMAX / TWO.lo, MAX);
        assert_eq!(ONE % MAX.lo, 1);
        assert_eq!(TWO % MAX.lo, 2);
        assert_eq!(ONEMAX % MAX.lo, 1);
        assert_eq!(ONEMAX % TWO.lo, 1);

        assert_eq!(ONEMAX.checked_mul1(MAX.lo), None);
        assert_eq!(TWOZERO.checked_mul1(MAX.lo), None);
    }

    #[test]
    fn test_assign_ops() {
        for _ in 0..10 {
            let x = udouble {
                hi: random::<u32>() as umax,
                lo: random(),
            };
            let y = udouble {
                hi: random::<u32>() as umax,
                lo: random(),
            };
            let mut z = x;

            z += y;
            assert_eq!(z, x + y);
            z -= y;
            assert_eq!(z, x);
        }
    }
}
