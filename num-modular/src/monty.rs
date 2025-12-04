use crate::reduced::impl_reduced_binary_pow;
use crate::{ModularUnaryOps, Reducer, Vanilla};

/// Negated modular inverse on binary bases
/// `neginv` calculates `-(m^-1) mod R`, `R = 2^k. If m is odd, then result of m + 1 will be returned.
mod neg_mod_inv {
    // Entry i contains (2i+1)^(-1) mod 256.
    #[rustfmt::skip]
    const BINV_TABLE: [u8; 128] = [
        0x01, 0xAB, 0xCD, 0xB7, 0x39, 0xA3, 0xC5, 0xEF, 0xF1, 0x1B, 0x3D, 0xA7, 0x29, 0x13, 0x35, 0xDF,
        0xE1, 0x8B, 0xAD, 0x97, 0x19, 0x83, 0xA5, 0xCF, 0xD1, 0xFB, 0x1D, 0x87, 0x09, 0xF3, 0x15, 0xBF,
        0xC1, 0x6B, 0x8D, 0x77, 0xF9, 0x63, 0x85, 0xAF, 0xB1, 0xDB, 0xFD, 0x67, 0xE9, 0xD3, 0xF5, 0x9F,
        0xA1, 0x4B, 0x6D, 0x57, 0xD9, 0x43, 0x65, 0x8F, 0x91, 0xBB, 0xDD, 0x47, 0xC9, 0xB3, 0xD5, 0x7F,
        0x81, 0x2B, 0x4D, 0x37, 0xB9, 0x23, 0x45, 0x6F, 0x71, 0x9B, 0xBD, 0x27, 0xA9, 0x93, 0xB5, 0x5F,
        0x61, 0x0B, 0x2D, 0x17, 0x99, 0x03, 0x25, 0x4F, 0x51, 0x7B, 0x9D, 0x07, 0x89, 0x73, 0x95, 0x3F,
        0x41, 0xEB, 0x0D, 0xF7, 0x79, 0xE3, 0x05, 0x2F, 0x31, 0x5B, 0x7D, 0xE7, 0x69, 0x53, 0x75, 0x1F,
        0x21, 0xCB, 0xED, 0xD7, 0x59, 0xC3, 0xE5, 0x0F, 0x11, 0x3B, 0x5D, 0xC7, 0x49, 0x33, 0x55, 0xFF,
    ];

    pub mod u8 {
        use super::*;
        pub const fn neginv(m: u8) -> u8 {
            let i = BINV_TABLE[((m >> 1) & 0x7F) as usize];
            i.wrapping_neg()
        }
    }

    pub mod u16 {
        use super::*;
        pub const fn neginv(m: u16) -> u16 {
            let mut i = BINV_TABLE[((m >> 1) & 0x7F) as usize] as u16;
            // hensel lifting
            i = 2u16.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i.wrapping_neg()
        }
    }

    pub mod u32 {
        use super::*;
        pub const fn neginv(m: u32) -> u32 {
            let mut i = BINV_TABLE[((m >> 1) & 0x7F) as usize] as u32;
            i = 2u32.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u32.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i.wrapping_neg()
        }
    }

    pub mod u64 {
        use super::*;
        pub const fn neginv(m: u64) -> u64 {
            let mut i = BINV_TABLE[((m >> 1) & 0x7F) as usize] as u64;
            i = 2u64.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u64.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u64.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i.wrapping_neg()
        }
    }

    pub mod u128 {
        use super::*;
        pub const fn neginv(m: u128) -> u128 {
            let mut i = BINV_TABLE[((m >> 1) & 0x7F) as usize] as u128;
            i = 2u128.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u128.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u128.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i = 2u128.wrapping_sub(i.wrapping_mul(m)).wrapping_mul(i);
            i.wrapping_neg()
        }
    }

    pub mod usize {
        #[inline]
        pub const fn neginv(m: usize) -> usize {
            #[cfg(target_pointer_width = "16")]
            return super::u16::neginv(m as _) as _;
            #[cfg(target_pointer_width = "32")]
            return super::u32::neginv(m as _) as _;
            #[cfg(target_pointer_width = "64")]
            return super::u64::neginv(m as _) as _;
        }
    }
}

/// A modular reducer based on [Montgomery form](https://en.wikipedia.org/wiki/Montgomery_modular_multiplication#Montgomery_form), only supports odd modulus.
///
/// The generic type T represents the underlying integer representation for modular inverse `-m^-1 mod R`,
/// and `R=2^B` will be used as the auxiliary modulus, where B is automatically selected
/// based on the size of T.
#[derive(Debug, Clone, Copy)]
pub struct Montgomery<T> {
    m: T,   // modulus
    inv: T, // modular inverse of the modulus
}

macro_rules! impl_montgomery_for {
    ($t:ident, $ns:ident) => {
        mod $ns {
            use super::*;
            use crate::word::$t::*;
            use neg_mod_inv::$t::neginv;

            impl Montgomery<$t> {
                pub const fn new(m: $t) -> Self {
                    assert!(
                        m & 1 != 0,
                        "Only odd modulus are supported by the Montgomery form"
                    );
                    Self { m, inv: neginv(m) }
                }
                const fn reduce(&self, monty: DoubleWord) -> $t {
                    debug_assert!(high(monty) < self.m);

                    // REDC algorithm
                    let tm = low(monty).wrapping_mul(self.inv);
                    let (t, overflow) = monty.overflowing_add(wmul(tm, self.m));
                    let t = high(t);

                    if overflow {
                        t + self.m.wrapping_neg()
                    } else if t >= self.m {
                        t - self.m
                    } else {
                        t
                    }
                }
            }

            impl Reducer<$t> for Montgomery<$t> {
                #[inline]
                fn new(m: &$t) -> Self {
                    Self::new(*m)
                }
                #[inline]
                fn transform(&self, target: $t) -> $t {
                    if target == 0 {
                        return 0;
                    }
                    nrem(merge(0, target), self.m)
                }
                #[inline]
                fn check(&self, target: &$t) -> bool {
                    *target < self.m
                }

                #[inline]
                fn residue(&self, target: $t) -> $t {
                    self.reduce(extend(target))
                }
                #[inline(always)]
                fn modulus(&self) -> $t {
                    self.m
                }
                #[inline(always)]
                fn is_zero(&self, target: &$t) -> bool {
                    *target == 0
                }

                #[inline(always)]
                fn add(&self, lhs: &$t, rhs: &$t) -> $t {
                    Vanilla::<$t>::add(&self.m, *lhs, *rhs)
                }

                #[inline(always)]
                fn dbl(&self, target: $t) -> $t {
                    Vanilla::<$t>::dbl(&self.m, target)
                }

                #[inline(always)]
                fn sub(&self, lhs: &$t, rhs: &$t) -> $t {
                    Vanilla::<$t>::sub(&self.m, *lhs, *rhs)
                }

                #[inline(always)]
                fn neg(&self, target: $t) -> $t {
                    Vanilla::<$t>::neg(&self.m, target)
                }

                #[inline]
                fn mul(&self, lhs: &$t, rhs: &$t) -> $t {
                    self.reduce(wmul(*lhs, *rhs))
                }

                #[inline]
                fn sqr(&self, target: $t) -> $t {
                    self.reduce(wsqr(target))
                }

                #[inline(always)]
                fn inv(&self, target: $t) -> Option<$t> {
                    // TODO: support direct montgomery inverse
                    // REF: http://cetinkayakoc.net/docs/j82.pdf
                    self.residue(target)
                        .invm(&self.m)
                        .map(|v| self.transform(v))
                }

                impl_reduced_binary_pow!(Word);
            }
        }
    };
}
impl_montgomery_for!(u8, u8_impl);
impl_montgomery_for!(u16, u16_impl);
impl_montgomery_for!(u32, u32_impl);
impl_montgomery_for!(u64, u64_impl);
impl_montgomery_for!(u128, u128_impl);
impl_montgomery_for!(usize, usize_impl);

// TODO(v0.6.x): accept even numbers by removing 2 factors from m and store the exponent
// Requirement: 1. A separate class to perform modular arithmetics with 2^n as modulus
//              2. Algorithm for construct residue from two components (see http://koclab.cs.ucsb.edu/teaching/cs154/docx/Notes7-Montgomery.pdf)
// Or we can just provide crt function, and let the implementation of monty int with full modulus support as an example code.

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    const NRANDOM: u32 = 10;

    #[test]
    fn creation_test() {
        // a deterministic test case for u128
        let a = (0x81u128 << 120) - 1;
        let m = (0x81u128 << 119) - 1;
        let m = m >> m.trailing_zeros();
        let r = Montgomery::<u128>::new(m);
        assert_eq!(r.residue(r.transform(a)), a % m);

        // is_zero test
        let r = Montgomery::<u8>::new(11u8);
        assert!(r.is_zero(&r.transform(0)));
        let five = r.transform(5u8);
        let six = r.transform(6u8);
        assert!(r.is_zero(&r.add(&five, &six)));

        // random creation test
        for _ in 0..NRANDOM {
            let a = random::<u8>();
            let m = random::<u8>() | 1;
            let r = Montgomery::<u8>::new(m);
            assert_eq!(r.residue(r.transform(a)), a % m);

            let a = random::<u16>();
            let m = random::<u16>() | 1;
            let r = Montgomery::<u16>::new(m);
            assert_eq!(r.residue(r.transform(a)), a % m);

            let a = random::<u32>();
            let m = random::<u32>() | 1;
            let r = Montgomery::<u32>::new(m);
            assert_eq!(r.residue(r.transform(a)), a % m);

            let a = random::<u64>();
            let m = random::<u64>() | 1;
            let r = Montgomery::<u64>::new(m);
            assert_eq!(r.residue(r.transform(a)), a % m);

            let a = random::<u128>();
            let m = random::<u128>() | 1;
            let r = Montgomery::<u128>::new(m);
            assert_eq!(r.residue(r.transform(a)), a % m);
        }
    }

    #[test]
    fn test_against_modops() {
        use crate::reduced::tests::ReducedTester;
        for _ in 0..NRANDOM {
            ReducedTester::<u8>::test_against_modops::<Montgomery<u8>>(true);
            ReducedTester::<u16>::test_against_modops::<Montgomery<u16>>(true);
            ReducedTester::<u32>::test_against_modops::<Montgomery<u32>>(true);
            ReducedTester::<u64>::test_against_modops::<Montgomery<u64>>(true);
            ReducedTester::<u128>::test_against_modops::<Montgomery<u128>>(true);
            ReducedTester::<usize>::test_against_modops::<Montgomery<usize>>(true);
        }
    }
}
