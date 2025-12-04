//! Implementations for modular operations on primitive integers

use crate::{udouble, Reducer, Vanilla};
use crate::{DivExact, ModularAbs, ModularCoreOps, ModularPow, ModularSymbols, ModularUnaryOps};

// FIXME: implement the modular functions as const after https://github.com/rust-lang/rust/pull/68847

macro_rules! impl_core_ops_uu {
    ($($T:ty => $Tdouble:ty;)*) => ($(
        impl ModularCoreOps<$T, &$T> for $T {
            type Output = $T;
            #[inline(always)]
            fn addm(self, rhs: $T, m: &$T) -> $T {
                (((self as $Tdouble) + (rhs as $Tdouble)) % (*m as $Tdouble)) as $T
            }
            #[inline]
            fn subm(self, rhs: $T, m: &$T) -> $T {
                if self >= rhs {
                    (self - rhs) % m
                } else {
                    ((rhs - self) % m).negm(m)
                }
            }
            #[inline(always)]
            fn mulm(self, rhs: $T, m: &$T) -> $T {
                (((self as $Tdouble) * (rhs as $Tdouble)) % (*m as $Tdouble)) as $T
            }
        }
    )*);
}
impl_core_ops_uu! { u8 => u16; u16 => u32; u32 => u64; u64 => u128; }

#[cfg(target_pointer_width = "16")]
impl_core_ops_uu! { usize => u32; }
#[cfg(target_pointer_width = "32")]
impl_core_ops_uu! { usize => u64; }
#[cfg(target_pointer_width = "64")]
impl_core_ops_uu! { usize => u128; }

impl ModularCoreOps<u128, &u128> for u128 {
    type Output = u128;

    #[inline]
    fn addm(self, rhs: u128, m: &u128) -> u128 {
        if let Some(ab) = self.checked_add(rhs) {
            ab % m
        } else {
            udouble::widening_add(self, rhs) % *m
        }
    }

    #[inline]
    fn subm(self, rhs: u128, m: &u128) -> u128 {
        if self >= rhs {
            (self - rhs) % m
        } else {
            ((rhs - self) % m).negm(m)
        }
    }

    #[inline]
    fn mulm(self, rhs: u128, m: &u128) -> u128 {
        if let Some(ab) = self.checked_mul(rhs) {
            ab % m
        } else {
            udouble::widening_mul(self, rhs) % *m
        }
    }
}

macro_rules! impl_powm_uprim {
    ($($T:ty)*) => ($(
        impl ModularPow<$T, &$T> for $T {
            type Output = $T;
            #[inline(always)]
            fn powm(self, exp: $T, m: &$T) -> $T {
                Vanilla::<$T>::new(&m).pow(self % m, &exp)
            }
        }
    )*);
}
impl_powm_uprim!(u8 u16 u32 u64 u128 usize);

macro_rules! impl_symbols_uprim {
    ($($T:ty)*) => ($(
        impl ModularSymbols<&$T> for $T {
            #[inline]
            fn checked_legendre(&self, n: &$T) -> Option<i8> {
                match self.powm((n - 1)/2, &n) {
                    0 => Some(0),
                    1 => Some(1),
                    x if x == n - 1 => Some(-1),
                    _ => None,
                }
            }

            fn checked_jacobi(&self, n: &$T) -> Option<i8> {
                if n % 2 == 0 {
                    return None;
                }
                if self == &0 {
                    return Some(if n == &1 {
                        1
                    } else {
                        0
                    });
                }
                if self == &1 {
                    return Some(1);
                }

                let mut a = self % n;
                let mut n = *n;
                let mut t = 1;
                while a > 0 {
                    while a % 2 == 0 {
                        a /= 2;
                        if n % 8 == 3 || n % 8 == 5 {
                            t *= -1;
                        }
                    }
                    core::mem::swap(&mut a, &mut n);
                    if a % 4 == 3 && n % 4 == 3 {
                        t *= -1;
                    }
                    a %= n;
                }
                Some(if n == 1 {
                    t
                } else {
                    0
                })
            }

            fn kronecker(&self, n: &$T) -> i8 {
                match n {
                    0 => {
                        if self == &1 {
                            1
                        } else {
                            0
                        }
                    }
                    1 => 1,
                    2 => {
                        if self % 2 == 0 {
                            0
                        } else if self % 8 == 1 || self % 8 == 7 {
                            1
                        } else {
                            -1
                        }
                    }
                    _ => {
                        let f = n.trailing_zeros();
                        let n = n >> f;
                        self.kronecker(&2).pow(f)
                            * self.jacobi(&n)
                    }
                }
            }
        }
    )*);
}
impl_symbols_uprim!(u8 u16 u32 u64 u128 usize);

macro_rules! impl_symbols_iprim {
    ($($T:ty, $U:ty;)*) => ($(
        impl ModularSymbols<&$T> for $T {
            #[inline]
            fn checked_legendre(&self, n: &$T) -> Option<i8> {
                if n < &1 {
                    return None;
                }
                let a = self.rem_euclid(*n) as $U;
                a.checked_legendre(&(*n as $U))
            }

            #[inline]
            fn checked_jacobi(&self, n: &$T) -> Option<i8> {
                if n < &1 {
                    return None;
                }
                let a = self.rem_euclid(*n) as $U;
                a.checked_jacobi(&(*n as $U))
            }

            #[inline]
            fn kronecker(&self, n: &$T) -> i8 {
                match n {
                    -1 => {
                        if self < &0 {
                            -1
                        } else {
                            1
                        }
                    }
                    0 => {
                        if self == &1 {
                            1
                        } else {
                            0
                        }
                    }
                    1 => 1,
                    2 => {
                        if self % 2 == 0 {
                            0
                        } else if self.rem_euclid(8) == 1 || self.rem_euclid(8) == 7 {
                            1
                        } else {
                            -1
                        }
                    },
                    i if i < &-1 => {
                        self.kronecker(&-1) * self.kronecker(&-i)
                    },
                    _ => {
                        let f = n.trailing_zeros();
                        self.kronecker(&2).pow(f)
                            * self.jacobi(&(n >> f))
                    }
                }
            }
        }
    )*);
}

impl_symbols_iprim!(i8, u8; i16, u16; i32, u32; i64, u64; i128, u128; isize, usize;);

macro_rules! impl_unary_uprim {
    ($($T:ty)*) => ($(
        impl ModularUnaryOps<&$T> for $T {
            type Output = $T;
            #[inline]
            fn negm(self, m: &$T) -> $T {
                let x = self % m;
                if x == 0 {
                    0
                } else {
                    m - x
                }
            }

            // inverse mod using extended euclidean algorithm
            fn invm(self, m: &$T) -> Option<$T> {
                // TODO: optimize using https://eprint.iacr.org/2020/972.pdf
                let x = if &self >= m { self % m } else { self.clone() };

                let (mut last_r, mut r) = (m.clone(), x);
                let (mut last_t, mut t) = (0, 1);

                while r > 0 {
                    let (quo, rem) = (last_r / r, last_r % r);
                    last_r = r;
                    r = rem;

                    let new_t = last_t.subm(quo.mulm(t, m), m);
                    last_t = t;
                    t = new_t;
                }

                // if r = gcd(self, m) > 1, then inverse doesn't exist
                if last_r > 1 {
                    None
                } else {
                    Some(last_t)
                }
            }

            #[inline(always)]
            fn dblm(self, m: &$T) -> $T {
                self.addm(self, m)
            }
            #[inline(always)]
            fn sqm(self, m: &$T) -> $T {
                self.mulm(self, m)
            }
        }
    )*);
}
impl_unary_uprim!(u8 u16 u32 u64 u128 usize);

// forward modular operations to valye by value
macro_rules! impl_mod_ops_by_deref {
    ($($T:ty)*) => {$(
        // core ops
        impl ModularCoreOps<$T, &$T> for &$T {
            type Output = $T;
            #[inline]
            fn addm(self, rhs: $T, m: &$T) -> $T {
                (*self).addm(rhs, &m)
            }
            #[inline]
            fn subm(self, rhs: $T, m: &$T) -> $T {
                (*self).subm(rhs, &m)
            }
            #[inline]
            fn mulm(self, rhs: $T, m: &$T) -> $T {
                (*self).mulm(rhs, &m)
            }
        }
        impl ModularCoreOps<&$T, &$T> for $T {
            type Output = $T;
            #[inline]
            fn addm(self, rhs: &$T, m: &$T) -> $T {
                self.addm(*rhs, &m)
            }
            #[inline]
            fn subm(self, rhs: &$T, m: &$T) -> $T {
                self.subm(*rhs, &m)
            }
            #[inline]
            fn mulm(self, rhs: &$T, m: &$T) -> $T {
                self.mulm(*rhs, &m)
            }
        }
        impl ModularCoreOps<&$T, &$T> for &$T {
            type Output = $T;
            #[inline]
            fn addm(self, rhs: &$T, m: &$T) -> $T {
                (*self).addm(*rhs, &m)
            }
            #[inline]
            fn subm(self, rhs: &$T, m: &$T) -> $T {
                (*self).subm(*rhs, &m)
            }
            #[inline]
            fn mulm(self, rhs: &$T, m: &$T) -> $T {
                (*self).mulm(*rhs, &m)
            }
        }

        // pow
        impl ModularPow<$T, &$T> for &$T {
            type Output = $T;
            #[inline]
            fn powm(self, exp: $T, m: &$T) -> $T {
                (*self).powm(exp, &m)
            }
        }
        impl ModularPow<&$T, &$T> for $T {
            type Output = $T;
            #[inline]
            fn powm(self, exp: &$T, m: &$T) -> $T {
                self.powm(*exp, &m)
            }
        }
        impl ModularPow<&$T, &$T> for &$T {
            type Output = $T;
            #[inline]
            fn powm(self, exp: &$T, m: &$T) -> $T {
                (*self).powm(*exp, &m)
            }
        }

        // unary ops
        impl ModularUnaryOps<&$T> for &$T {
            type Output = $T;

            #[inline]
            fn negm(self, m: &$T) -> $T {
                ModularUnaryOps::<&$T>::negm(*self, m)
            }
            #[inline]
            fn invm(self, m: &$T) -> Option<$T> {
                ModularUnaryOps::<&$T>::invm(*self, m)
            }
            #[inline]
            fn dblm(self, m: &$T) -> $T {
                ModularUnaryOps::<&$T>::dblm(*self, m)
            }
            #[inline]
            fn sqm(self, m: &$T) -> $T {
                ModularUnaryOps::<&$T>::sqm(*self, m)
            }
        }
    )*};
}

impl_mod_ops_by_deref!(u8 u16 u32 u64 u128 usize);

macro_rules! impl_absm_for_prim {
    ($($signed:ty => $unsigned:ty;)*) => {$(
        impl ModularAbs<$unsigned> for $signed {
            fn absm(self, m: &$unsigned) -> $unsigned {
                if self >= 0 {
                    (self as $unsigned) % m
                } else {
                    (-self as $unsigned).negm(m)
                }
            }
        }
    )*};
}

impl_absm_for_prim! {
    i8 => u8; i16 => u16; i32 => u32; i64 => u64; i128 => u128; isize => usize;
}

macro_rules! impl_div_exact_for_prim {
    ($($t:ty)*) => {$(
        impl DivExact<$t, ()> for $t {
            type Output = $t;
            #[inline]
            fn div_exact(self, d: $t, _: &()) -> Option<Self::Output> {
                let (q, r) = (self / d, self % d);
                if r == 0 {
                    Some(q)
                } else {
                    None
                }
            }
        }
    )*};
}

impl_div_exact_for_prim!(u8 u16 u32 u64 u128);

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::Neg;
    use rand::random;

    const NRANDOM: u32 = 10; // number of random tests to run

    #[test]
    fn addm_test() {
        // fixed cases
        const CASES: [(u8, u8, u8, u8); 10] = [
            // [m, x, y, rem]: x + y = rem (mod m)
            (5, 0, 0, 0),
            (5, 1, 2, 3),
            (5, 2, 1, 3),
            (5, 2, 2, 4),
            (5, 3, 2, 0),
            (5, 2, 3, 0),
            (5, 6, 1, 2),
            (5, 1, 6, 2),
            (5, 11, 7, 3),
            (5, 7, 11, 3),
        ];

        for &(m, x, y, r) in CASES.iter() {
            assert_eq!(x.addm(y, &m), r);
            assert_eq!((x as u16).addm(y as u16, &(m as _)), r as _);
            assert_eq!((x as u32).addm(y as u32, &(m as _)), r as _);
            assert_eq!((x as u64).addm(y as u64, &(m as _)), r as _);
            assert_eq!((x as u128).addm(y as u128, &(m as _)), r as _);
        }

        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u32>() as u64;
            let b = random::<u32>() as u64;
            let m = random::<u32>() as u64;
            assert_eq!(a.addm(b, &m), (a + b) % m);
            assert_eq!(
                a.addm(b, &(1u64 << 32)) as u32,
                (a as u32).wrapping_add(b as u32)
            );

            let a = random::<u64>() as u128;
            let b = random::<u64>() as u128;
            let m = random::<u64>() as u128;
            assert_eq!(a.addm(b, &m), (a + b) % m);
            assert_eq!(
                a.addm(b, &(1u128 << 64)) as u64,
                (a as u64).wrapping_add(b as u64)
            );
        }
    }

    #[test]
    fn subm_test() {
        // fixed cases
        const CASES: [(u8, u8, u8, u8); 10] = [
            // [m, x, y, rem]: x - y = rem (mod m)
            (7, 0, 0, 0),
            (7, 11, 9, 2),
            (7, 5, 2, 3),
            (7, 2, 5, 4),
            (7, 6, 7, 6),
            (7, 1, 7, 1),
            (7, 7, 1, 6),
            (7, 0, 6, 1),
            (7, 15, 1, 0),
            (7, 1, 15, 0),
        ];

        for &(m, x, y, r) in CASES.iter() {
            assert_eq!(x.subm(y, &m), r);
            assert_eq!((x as u16).subm(y as u16, &(m as _)), r as _);
            assert_eq!((x as u32).subm(y as u32, &(m as _)), r as _);
            assert_eq!((x as u64).subm(y as u64, &(m as _)), r as _);
            assert_eq!((x as u128).subm(y as u128, &(m as _)), r as _);
        }

        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u32>() as u64;
            let b = random::<u32>() as u64;
            let m = random::<u32>() as u64;
            assert_eq!(
                a.subm(b, &m),
                (a as i64 - b as i64).rem_euclid(m as i64) as u64
            );
            assert_eq!(
                a.subm(b, &(1u64 << 32)) as u32,
                (a as u32).wrapping_sub(b as u32)
            );

            let a = random::<u64>() as u128;
            let b = random::<u64>() as u128;
            let m = random::<u64>() as u128;
            assert_eq!(
                a.subm(b, &m),
                (a as i128 - b as i128).rem_euclid(m as i128) as u128
            );
            assert_eq!(
                a.subm(b, &(1u128 << 64)) as u64,
                (a as u64).wrapping_sub(b as u64)
            );
        }
    }

    #[test]
    fn negm_and_absm_test() {
        // fixed cases
        const CASES: [(u8, u8, u8); 5] = [
            // [m, x, rem]: -x = rem (mod m)
            (5, 0, 0),
            (5, 2, 3),
            (5, 1, 4),
            (5, 5, 0),
            (5, 12, 3),
        ];

        for &(m, x, r) in CASES.iter() {
            assert_eq!(x.negm(&m), r);
            assert_eq!((x as i8).neg().absm(&m), r);
            assert_eq!((x as u16).negm(&(m as _)), r as _);
            assert_eq!((x as i16).neg().absm(&(m as u16)), r as _);
            assert_eq!((x as u32).negm(&(m as _)), r as _);
            assert_eq!((x as i32).neg().absm(&(m as u32)), r as _);
            assert_eq!((x as u64).negm(&(m as _)), r as _);
            assert_eq!((x as i64).neg().absm(&(m as u64)), r as _);
            assert_eq!((x as u128).negm(&(m as _)), r as _);
            assert_eq!((x as i128).neg().absm(&(m as u128)), r as _);
        }

        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u32>() as u64;
            let m = random::<u32>() as u64;
            assert_eq!(a.negm(&m), (a as i64).neg().rem_euclid(m as i64) as u64);
            assert_eq!(a.negm(&(1u64 << 32)) as u32, (a as u32).wrapping_neg());

            let a = random::<u64>() as u128;
            let m = random::<u64>() as u128;
            assert_eq!(a.negm(&m), (a as i128).neg().rem_euclid(m as i128) as u128);
            assert_eq!(a.negm(&(1u128 << 64)) as u64, (a as u64).wrapping_neg());
        }
    }

    #[test]
    fn mulm_test() {
        // fixed cases
        const CASES: [(u8, u8, u8, u8); 10] = [
            // [m, x, y, rem]: x*y = rem (mod m)
            (7, 0, 0, 0),
            (7, 11, 9, 1),
            (7, 5, 2, 3),
            (7, 2, 5, 3),
            (7, 6, 7, 0),
            (7, 1, 7, 0),
            (7, 7, 1, 0),
            (7, 0, 6, 0),
            (7, 15, 1, 1),
            (7, 1, 15, 1),
        ];

        for &(m, x, y, r) in CASES.iter() {
            assert_eq!(x.mulm(y, &m), r);
            assert_eq!((x as u16).mulm(y as u16, &(m as _)), r as _);
            assert_eq!((x as u32).mulm(y as u32, &(m as _)), r as _);
            assert_eq!((x as u64).mulm(y as u64, &(m as _)), r as _);
            assert_eq!((x as u128).mulm(y as u128, &(m as _)), r as _);
        }

        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u32>() as u64;
            let b = random::<u32>() as u64;
            let m = random::<u32>() as u64;
            assert_eq!(a.mulm(b, &m), (a * b) % m);
            assert_eq!(
                a.mulm(b, &(1u64 << 32)) as u32,
                (a as u32).wrapping_mul(b as u32)
            );

            let a = random::<u64>() as u128;
            let b = random::<u64>() as u128;
            let m = random::<u64>() as u128;
            assert_eq!(a.mulm(b, &m), (a * b) % m);
            assert_eq!(
                a.mulm(b, &(1u128 << 32)) as u32,
                (a as u32).wrapping_mul(b as u32)
            );
        }
    }

    #[test]
    fn powm_test() {
        // fixed cases
        const CASES: [(u8, u8, u8, u8); 12] = [
            // [m, x, y, rem]: x^y = rem (mod m)
            (7, 0, 0, 1),
            (7, 11, 9, 1),
            (7, 5, 2, 4),
            (7, 2, 5, 4),
            (7, 6, 7, 6),
            (7, 1, 7, 1),
            (7, 7, 1, 0),
            (7, 0, 6, 0),
            (7, 15, 1, 1),
            (7, 1, 15, 1),
            (7, 255, 255, 6),
            (10, 255, 255, 5),
        ];

        for &(m, x, y, r) in CASES.iter() {
            assert_eq!(x.powm(y, &m), r);
            assert_eq!((x as u16).powm(y as u16, &(m as _)), r as _);
            assert_eq!((x as u32).powm(y as u32, &(m as _)), r as _);
            assert_eq!((x as u64).powm(y as u64, &(m as _)), r as _);
            assert_eq!((x as u128).powm(y as u128, &(m as _)), r as _);
        }
    }

    #[test]
    fn invm_test() {
        // fixed cases
        const CASES: [(u64, u64, u64); 8] = [
            // [a, m, x] s.t. a*x = 1 (mod m) is satisfied
            (5, 11, 9),
            (8, 11, 7),
            (10, 11, 10),
            (3, 5000, 1667),
            (1667, 5000, 3),
            (999, 5000, 3999),
            (999, 9_223_372_036_854_775_807, 3_619_181_019_466_538_655),
            (
                9_223_372_036_854_775_804,
                9_223_372_036_854_775_807,
                3_074_457_345_618_258_602,
            ),
        ];

        for &(a, m, x) in CASES.iter() {
            assert_eq!(a.invm(&m).unwrap(), x);
        }

        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u32>() as u64;
            let m = random::<u32>() as u64;
            if let Some(ia) = a.invm(&m) {
                assert_eq!(a.mulm(ia, &m), 1);
            }

            let a = random::<u64>() as u128;
            let m = random::<u64>() as u128;
            if let Some(ia) = a.invm(&m) {
                assert_eq!(a.mulm(ia, &m), 1);
            }
        }
    }

    #[test]
    fn dblm_and_sqm_test() {
        // random cases for u64 and u128
        for _ in 0..NRANDOM {
            let a = random::<u64>();
            let m = random::<u64>();
            assert_eq!(a.addm(a, &m), a.dblm(&m));
            assert_eq!(a.mulm(2, &m), a.dblm(&m));
            assert_eq!(a.mulm(a, &m), a.sqm(&m));
            assert_eq!(a.powm(2, &m), a.sqm(&m));

            let a = random::<u128>();
            let m = random::<u128>();
            assert_eq!(a.addm(a, &m), a.dblm(&m));
            assert_eq!(a.mulm(2, &m), a.dblm(&m));
            assert_eq!(a.mulm(a, &m), a.sqm(&m));
            assert_eq!(a.powm(2, &m), a.sqm(&m));
        }
    }

    #[test]
    fn legendre_test() {
        const CASES: [(u8, u8, i8); 18] = [
            (0, 11, 0),
            (1, 11, 1),
            (2, 11, -1),
            (4, 11, 1),
            (7, 11, -1),
            (10, 11, -1),
            (0, 17, 0),
            (1, 17, 1),
            (2, 17, 1),
            (4, 17, 1),
            (9, 17, 1),
            (10, 17, -1),
            (0, 101, 0),
            (1, 101, 1),
            (2, 101, -1),
            (4, 101, 1),
            (9, 101, 1),
            (10, 101, -1),
        ];

        for &(a, n, res) in CASES.iter() {
            assert_eq!(a.legendre(&n), res);
            assert_eq!((a as u16).legendre(&(n as u16)), res);
            assert_eq!((a as u32).legendre(&(n as u32)), res);
            assert_eq!((a as u64).legendre(&(n as u64)), res);
            assert_eq!((a as u128).legendre(&(n as u128)), res);
        }

        const SIGNED_CASES: [(i8, i8, i8); 15] = [
            (-10, 11, 1),
            (-7, 11, 1),
            (-4, 11, -1),
            (-2, 11, 1),
            (-1, 11, -1),
            (-10, 17, -1),
            (-9, 17, 1),
            (-4, 17, 1),
            (-2, 17, 1),
            (-1, 17, 1),
            (-10, 101, -1),
            (-9, 101, 1),
            (-4, 101, 1),
            (-2, 101, -1),
            (-1, 101, 1),
        ];

        for &(a, n, res) in SIGNED_CASES.iter() {
            assert_eq!(a.legendre(&n), res);
            assert_eq!((a as i16).legendre(&(n as i16)), res);
            assert_eq!((a as i32).legendre(&(n as i32)), res);
            assert_eq!((a as i64).legendre(&(n as i64)), res);
            assert_eq!((a as i128).legendre(&(n as i128)), res);
        }
    }

    #[test]
    fn jacobi_test() {
        const CASES: [(u8, u8, i8); 15] = [
            (1, 1, 1),
            (15, 1, 1),
            (2, 3, -1),
            (29, 9, 1),
            (4, 11, 1),
            (17, 11, -1),
            (19, 29, -1),
            (10, 33, -1),
            (11, 33, 0),
            (12, 33, 0),
            (14, 33, -1),
            (15, 33, 0),
            (15, 37, -1),
            (29, 59, 1),
            (30, 59, -1),
        ];

        for &(a, n, res) in CASES.iter() {
            assert_eq!(a.jacobi(&n), res, "{}, {}", a, n);
            assert_eq!((a as u16).jacobi(&(n as u16)), res);
            assert_eq!((a as u32).jacobi(&(n as u32)), res);
            assert_eq!((a as u64).jacobi(&(n as u64)), res);
            assert_eq!((a as u128).jacobi(&(n as u128)), res);
        }

        const SIGNED_CASES: [(i8, i8, i8); 15] = [
            (-10, 15, 0),
            (-7, 15, 1),
            (-4, 15, -1),
            (-2, 15, -1),
            (-1, 15, -1),
            (-10, 13, 1),
            (-9, 13, 1),
            (-4, 13, 1),
            (-2, 13, -1),
            (-1, 13, 1),
            (-10, 11, 1),
            (-9, 11, -1),
            (-4, 11, -1),
            (-2, 11, 1),
            (-1, 11, -1),
        ];

        for &(a, n, res) in SIGNED_CASES.iter() {
            assert_eq!(a.jacobi(&n), res);
            assert_eq!((a as i16).jacobi(&(n as i16)), res);
            assert_eq!((a as i32).jacobi(&(n as i32)), res);
            assert_eq!((a as i64).jacobi(&(n as i64)), res);
            assert_eq!((a as i128).jacobi(&(n as i128)), res);
        }
    }

    #[test]
    fn kronecker_test() {
        const CASES: [(u8, u8, i8); 18] = [
            (0, 15, 0),
            (1, 15, 1),
            (2, 15, 1),
            (4, 15, 1),
            (7, 15, -1),
            (10, 15, 0),
            (0, 14, 0),
            (1, 14, 1),
            (2, 14, 0),
            (4, 14, 0),
            (9, 14, 1),
            (10, 14, 0),
            (0, 11, 0),
            (1, 11, 1),
            (2, 11, -1),
            (4, 11, 1),
            (9, 11, 1),
            (10, 11, -1),
        ];

        for &(a, n, res) in CASES.iter() {
            assert_eq!(a.kronecker(&n), res);
            assert_eq!((a as u16).kronecker(&(n as u16)), res);
            assert_eq!((a as u32).kronecker(&(n as u32)), res);
            assert_eq!((a as u64).kronecker(&(n as u64)), res);
            assert_eq!((a as u128).kronecker(&(n as u128)), res);
        }

        const SIGNED_CASES: [(i8, i8, i8); 37] = [
            (-10, 15, 0),
            (-7, 15, 1),
            (-4, 15, -1),
            (-2, 15, -1),
            (-1, 15, -1),
            (-10, 14, 0),
            (-9, 14, -1),
            (-4, 14, 0),
            (-2, 14, 0),
            (-1, 14, -1),
            (-10, 11, 1),
            (-9, 11, -1),
            (-4, 11, -1),
            (-2, 11, 1),
            (-1, 11, -1),
            (-10, -11, -1),
            (-9, -11, 1),
            (-4, -11, 1),
            (-2, -11, -1),
            (-1, -11, 1),
            (0, -11, 0),
            (1, -11, 1),
            (2, -11, -1),
            (4, -11, 1),
            (9, -11, 1),
            (10, -11, -1),
            (-10, 32, 0),
            (-9, 32, 1),
            (-4, 32, 0),
            (-2, 32, 0),
            (-1, 32, 1),
            (0, 32, 0),
            (1, 32, 1),
            (2, 32, 0),
            (4, 32, 0),
            (9, 32, 1),
            (10, 32, 0),
        ];

        for &(a, n, res) in SIGNED_CASES.iter() {
            assert_eq!(a.kronecker(&n), res, "{}, {}", a, n);
            assert_eq!((a as i16).kronecker(&(n as i16)), res);
            assert_eq!((a as i32).kronecker(&(n as i32)), res);
            assert_eq!((a as i64).kronecker(&(n as i64)), res);
            assert_eq!((a as i128).kronecker(&(n as i128)), res);
        }
    }
}
