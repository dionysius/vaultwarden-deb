use crate::reduced::impl_reduced_binary_pow;
use crate::{udouble, umax, ModularUnaryOps, Reducer};

// FIXME: use unchecked operators to speed up calculation (after https://github.com/rust-lang/rust/issues/85122)
/// A modular reducer for (pseudo) Mersenne numbers `2^P - K` as modulus. It supports `P` up to 127 and `K < 2^(P-1)`
///
/// The `P` is limited to 127 so that it's not necessary to check overflow. This limit won't be a problem for any
/// Mersenne primes within the range of [umax] (i.e. [u128]).
#[derive(Debug, Clone, Copy)]
pub struct FixedMersenne<const P: u8, const K: umax>();

// XXX: support other primes as modulo, such as solinas prime, proth prime and support multi precision
// REF: Handbook of Cryptography 14.3.4

impl<const P: u8, const K: umax> FixedMersenne<P, K> {
    const BITMASK: umax = (1 << P) - 1;
    pub const MODULUS: umax = (1 << P) - K;

    // Calculate v % Self::MODULUS, where v is a umax integer
    const fn reduce_single(v: umax) -> umax {
        let mut lo = v & Self::BITMASK;
        let mut hi = v >> P;
        while hi > 0 {
            let sum = if K == 1 { hi + lo } else { hi * K + lo };
            lo = sum & Self::BITMASK;
            hi = sum >> P;
        }

        if lo >= Self::MODULUS {
            lo - Self::MODULUS
        } else {
            lo
        }
    }

    // Calculate v % Self::MODULUS, where v is a udouble integer
    fn reduce_double(v: udouble) -> umax {
        // reduce modulo
        let mut lo = v.lo & Self::BITMASK;
        let mut hi = v >> P;
        while hi.hi > 0 {
            // first reduce until high bits fit in umax
            let sum = if K == 1 { hi + lo } else { hi * K + lo };
            lo = sum.lo & Self::BITMASK;
            hi = sum >> P;
        }

        let mut hi = hi.lo;
        while hi > 0 {
            // then reduce the smaller high bits
            let sum = if K == 1 { hi + lo } else { hi * K + lo };
            lo = sum & Self::BITMASK;
            hi = sum >> P;
        }

        if lo >= Self::MODULUS {
            lo - Self::MODULUS
        } else {
            lo
        }
    }
}

impl<const P: u8, const K: umax> Reducer<umax> for FixedMersenne<P, K> {
    #[inline]
    fn new(m: &umax) -> Self {
        assert!(
            *m == Self::MODULUS,
            "the given modulus doesn't match with the generic params"
        );
        debug_assert!(P <= 127);
        debug_assert!(K > 0 && K < (2 as umax).pow(P as u32 - 1) && K % 2 == 1);
        debug_assert!(
            Self::MODULUS % 3 != 0
                && Self::MODULUS % 5 != 0
                && Self::MODULUS % 7 != 0
                && Self::MODULUS % 11 != 0
                && Self::MODULUS % 13 != 0
        ); // error on easy composites
        Self {}
    }
    #[inline]
    fn transform(&self, target: umax) -> umax {
        Self::reduce_single(target)
    }
    fn check(&self, target: &umax) -> bool {
        *target < Self::MODULUS
    }
    #[inline]
    fn residue(&self, target: umax) -> umax {
        target
    }
    #[inline]
    fn modulus(&self) -> umax {
        Self::MODULUS
    }
    #[inline]
    fn is_zero(&self, target: &umax) -> bool {
        target == &0
    }

    #[inline]
    fn add(&self, lhs: &umax, rhs: &umax) -> umax {
        let mut sum = lhs + rhs;
        if sum >= Self::MODULUS {
            sum -= Self::MODULUS
        }
        sum
    }
    #[inline]
    fn sub(&self, lhs: &umax, rhs: &umax) -> umax {
        if lhs >= rhs {
            lhs - rhs
        } else {
            Self::MODULUS - (rhs - lhs)
        }
    }
    #[inline]
    fn dbl(&self, target: umax) -> umax {
        self.add(&target, &target)
    }
    #[inline]
    fn neg(&self, target: umax) -> umax {
        if target == 0 {
            0
        } else {
            Self::MODULUS - target
        }
    }
    #[inline]
    fn mul(&self, lhs: &umax, rhs: &umax) -> umax {
        if (P as u32) < (umax::BITS / 2) {
            Self::reduce_single(lhs * rhs)
        } else {
            Self::reduce_double(udouble::widening_mul(*lhs, *rhs))
        }
    }
    #[inline]
    fn inv(&self, target: umax) -> Option<umax> {
        if (P as u32) < usize::BITS {
            (target as usize)
                .invm(&(Self::MODULUS as usize))
                .map(|v| v as umax)
        } else {
            target.invm(&Self::MODULUS)
        }
    }
    #[inline]
    fn sqr(&self, target: umax) -> umax {
        if (P as u32) < (umax::BITS / 2) {
            Self::reduce_single(target * target)
        } else {
            Self::reduce_double(udouble::widening_square(target))
        }
    }

    impl_reduced_binary_pow!(umax);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ModularCoreOps, ModularPow};
    use rand::random;

    type M1 = FixedMersenne<31, 1>;
    type M2 = FixedMersenne<61, 1>;
    type M3 = FixedMersenne<127, 1>;
    type M4 = FixedMersenne<32, 5>;
    type M5 = FixedMersenne<56, 5>;
    type M6 = FixedMersenne<122, 3>;

    const NRANDOM: u32 = 10;

    #[test]
    fn creation_test() {
        // random creation test
        for _ in 0..NRANDOM {
            let a = random::<umax>();

            const P1: umax = (1 << 31) - 1;
            let m1 = M1::new(&P1);
            assert_eq!(m1.residue(m1.transform(a)), a % P1);
            const P2: umax = (1 << 61) - 1;
            let m2 = M2::new(&P2);
            assert_eq!(m2.residue(m2.transform(a)), a % P2);
            const P3: umax = (1 << 127) - 1;
            let m3 = M3::new(&P3);
            assert_eq!(m3.residue(m3.transform(a)), a % P3);
            const P4: umax = (1 << 32) - 5;
            let m4 = M4::new(&P4);
            assert_eq!(m4.residue(m4.transform(a)), a % P4);
            const P5: umax = (1 << 56) - 5;
            let m5 = M5::new(&P5);
            assert_eq!(m5.residue(m5.transform(a)), a % P5);
            const P6: umax = (1 << 122) - 3;
            let m6 = M6::new(&P6);
            assert_eq!(m6.residue(m6.transform(a)), a % P6);
        }
    }

    #[test]
    fn test_against_modops() {
        macro_rules! tests_for {
            ($a:tt, $b:tt, $e:tt; $($M:ty)*) => ($({
                const P: umax = <$M>::MODULUS;
                let r = <$M>::new(&P);
                let am = r.transform($a);
                let bm = r.transform($b);
                assert_eq!(r.add(&am, &bm), $a.addm($b, &P));
                assert_eq!(r.sub(&am, &bm), $a.subm($b, &P));
                assert_eq!(r.mul(&am, &bm), $a.mulm($b, &P));
                assert_eq!(r.neg(am), $a.negm(&P));
                assert_eq!(r.inv(am), $a.invm(&P));
                assert_eq!(r.dbl(am), $a.dblm(&P));
                assert_eq!(r.sqr(am), $a.sqm(&P));
                assert_eq!(r.pow(am, &$e), $a.powm($e, &P));
            })*);
        }

        for _ in 0..NRANDOM {
            let (a, b) = (random::<u128>(), random::<u128>());
            let e = random::<u8>() as umax;
            tests_for!(a, b, e; M1 M2 M3 M4 M5 M6);
        }
    }
}
