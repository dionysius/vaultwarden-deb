use crate::{udouble, ModularInteger, ModularUnaryOps, Reducer};
use core::ops::*;
#[cfg(feature = "num_traits")]
use num_traits::{Inv, Pow};

/// An integer in a modulo ring
#[derive(Debug, Clone, Copy)]
pub struct ReducedInt<T, R: Reducer<T>> {
    /// The reduced representation of the integer in a modulo ring.
    a: T,

    /// The reducer for the integer
    r: R,
}

impl<T, R: Reducer<T>> ReducedInt<T, R> {
    /// Convert n into the modulo ring ℤ/mℤ (i.e. `n % m`)
    #[inline]
    pub fn new(n: T, m: &T) -> Self {
        let r = R::new(m);
        let a = r.transform(n);
        Self { a, r }
    }

    #[inline(always)]
    fn check_modulus_eq(&self, rhs: &Self)
    where
        T: PartialEq,
    {
        // we don't directly compare m because m could be empty in case of Mersenne modular integer
        if cfg!(debug_assertions) && self.r.modulus() != rhs.r.modulus() {
            panic!("The modulus of two operators should be the same!");
        }
    }

    #[inline(always)]
    pub fn repr(&self) -> &T {
        &self.a
    }

    #[inline(always)]
    pub fn inv(self) -> Option<Self> {
        Some(Self {
            a: self.r.inv(self.a)?,
            r: self.r,
        })
    }

    #[inline(always)]
    pub fn pow(self, exp: &T) -> Self {
        Self {
            a: self.r.pow(self.a, exp),
            r: self.r,
        }
    }
}

impl<T: PartialEq, R: Reducer<T>> PartialEq for ReducedInt<T, R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.check_modulus_eq(other);
        self.a == other.a
    }
}

macro_rules! impl_binops {
    ($method:ident, impl $op:ident) => {
        impl<T: PartialEq, R: Reducer<T>> $op for ReducedInt<T, R> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                self.check_modulus_eq(&rhs);
                let Self { a, r } = self;
                let a = r.$method(&a, &rhs.a);
                Self { a, r }
            }
        }

        impl<T: PartialEq + Clone, R: Reducer<T>> $op<&Self> for ReducedInt<T, R> {
            type Output = Self;
            #[inline]
            fn $method(self, rhs: &Self) -> Self::Output {
                self.check_modulus_eq(&rhs);
                let Self { a, r } = self;
                let a = r.$method(&a, &rhs.a);
                Self { a, r }
            }
        }

        impl<T: PartialEq + Clone, R: Reducer<T>> $op<ReducedInt<T, R>> for &ReducedInt<T, R> {
            type Output = ReducedInt<T, R>;
            #[inline]
            fn $method(self, rhs: ReducedInt<T, R>) -> Self::Output {
                self.check_modulus_eq(&rhs);
                let ReducedInt { a, r } = rhs;
                let a = r.$method(&self.a, &a);
                ReducedInt { a, r }
            }
        }

        impl<T: PartialEq + Clone, R: Reducer<T> + Clone> $op<&ReducedInt<T, R>>
            for &ReducedInt<T, R>
        {
            type Output = ReducedInt<T, R>;
            #[inline]
            fn $method(self, rhs: &ReducedInt<T, R>) -> Self::Output {
                self.check_modulus_eq(&rhs);
                let a = self.r.$method(&self.a, &rhs.a);
                ReducedInt {
                    a,
                    r: self.r.clone(),
                }
            }
        }

        impl<T: PartialEq, R: Reducer<T>> $op<T> for ReducedInt<T, R> {
            type Output = Self;
            fn $method(self, rhs: T) -> Self::Output {
                let Self { a, r } = self;
                let rhs = r.transform(rhs);
                let a = r.$method(&a, &rhs);
                Self { a, r }
            }
        }
    };
}
impl_binops!(add, impl Add);
impl_binops!(sub, impl Sub);
impl_binops!(mul, impl Mul);

impl<T: PartialEq, R: Reducer<T>> Neg for ReducedInt<T, R> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        let Self { a, r } = self;
        let a = r.neg(a);
        Self { a, r }
    }
}
impl<T: PartialEq + Clone, R: Reducer<T> + Clone> Neg for &ReducedInt<T, R> {
    type Output = ReducedInt<T, R>;
    #[inline]
    fn neg(self) -> Self::Output {
        let a = self.r.neg(self.a.clone());
        ReducedInt {
            a,
            r: self.r.clone(),
        }
    }
}

const INV_ERR_MSG: &str = "the modular inverse doesn't exist!";

#[cfg(feature = "num_traits")]
impl<T: PartialEq, R: Reducer<T>> Inv for ReducedInt<T, R> {
    type Output = Self;
    #[inline]
    fn inv(self) -> Self::Output {
        self.inv().expect(INV_ERR_MSG)
    }
}
#[cfg(feature = "num_traits")]
impl<T: PartialEq + Clone, R: Reducer<T> + Clone> Inv for &ReducedInt<T, R> {
    type Output = ReducedInt<T, R>;
    #[inline]
    fn inv(self) -> Self::Output {
        self.clone().inv().expect(INV_ERR_MSG)
    }
}

impl<T: PartialEq, R: Reducer<T>> Div for ReducedInt<T, R> {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.check_modulus_eq(&rhs);
        let ReducedInt { a, r } = rhs;
        let a = r.mul(&self.a, &r.inv(a).expect(INV_ERR_MSG));
        ReducedInt { a, r }
    }
}
impl<T: PartialEq + Clone, R: Reducer<T>> Div<&ReducedInt<T, R>> for ReducedInt<T, R> {
    type Output = Self;
    #[inline]
    fn div(self, rhs: &Self) -> Self::Output {
        self.check_modulus_eq(rhs);
        let Self { a, r } = self;
        let a = r.mul(&a, &r.inv(rhs.a.clone()).expect(INV_ERR_MSG));
        ReducedInt { a, r }
    }
}
impl<T: PartialEq + Clone, R: Reducer<T>> Div<ReducedInt<T, R>> for &ReducedInt<T, R> {
    type Output = ReducedInt<T, R>;
    #[inline]
    fn div(self, rhs: ReducedInt<T, R>) -> Self::Output {
        self.check_modulus_eq(&rhs);
        let ReducedInt { a, r } = rhs;
        let a = r.mul(&self.a, &r.inv(a).expect(INV_ERR_MSG));
        ReducedInt { a, r }
    }
}
impl<T: PartialEq + Clone, R: Reducer<T> + Clone> Div<&ReducedInt<T, R>> for &ReducedInt<T, R> {
    type Output = ReducedInt<T, R>;
    #[inline]
    fn div(self, rhs: &ReducedInt<T, R>) -> Self::Output {
        self.check_modulus_eq(rhs);
        let a = self
            .r
            .mul(&self.a, &self.r.inv(rhs.a.clone()).expect(INV_ERR_MSG));
        ReducedInt {
            a,
            r: self.r.clone(),
        }
    }
}

#[cfg(feature = "num_traits")]
impl<T: PartialEq, R: Reducer<T>> Pow<T> for ReducedInt<T, R> {
    type Output = Self;
    #[inline]
    fn pow(self, rhs: T) -> Self::Output {
        ReducedInt::pow(self, rhs)
    }
}
#[cfg(feature = "num_traits")]
impl<T: PartialEq + Clone, R: Reducer<T> + Clone> Pow<T> for &ReducedInt<T, R> {
    type Output = ReducedInt<T, R>;
    #[inline]
    fn pow(self, rhs: T) -> Self::Output {
        let a = self.r.pow(self.a.clone(), rhs);
        ReducedInt {
            a,
            r: self.r.clone(),
        }
    }
}

impl<T: PartialEq + Clone, R: Reducer<T> + Clone> ModularInteger for ReducedInt<T, R> {
    type Base = T;

    #[inline]
    fn modulus(&self) -> T {
        self.r.modulus()
    }

    #[inline(always)]
    fn residue(&self) -> T {
        debug_assert!(self.r.check(&self.a));
        self.r.residue(self.a.clone())
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.r.is_zero(&self.a)
    }

    #[inline]
    fn convert(&self, n: T) -> Self {
        Self {
            a: self.r.transform(n),
            r: self.r.clone(),
        }
    }

    #[inline]
    fn double(self) -> Self {
        let Self { a, r } = self;
        let a = r.dbl(a);
        Self { a, r }
    }

    #[inline]
    fn square(self) -> Self {
        let Self { a, r } = self;
        let a = r.sqr(a);
        Self { a, r }
    }
}

// An vanilla reducer is also provided here
/// A plain reducer that just use normal [Rem] operators. It will keep the integer
/// in range [0, modulus) after each operation.
#[derive(Debug, Clone, Copy)]
pub struct Vanilla<T>(T);

macro_rules! impl_uprim_vanilla_core_const {
    ($($T:ty)*) => {$(
        // These methods are for internal use only, wait for the introduction of const Trait in Rust
        impl Vanilla<$T> {
            #[inline]
            pub(crate) const fn add(m: &$T, lhs: $T, rhs: $T) -> $T {
                let (sum, overflow) = lhs.overflowing_add(rhs);
                if overflow || sum >= *m {
                    let (sum2, overflow2) = sum.overflowing_sub(*m);
                    debug_assert!(overflow == overflow2);
                    sum2
                } else {
                    sum
                }
            }

            #[inline]
            pub(crate) const fn dbl(m: &$T, target: $T) -> $T {
                Self::add(m, target, target)
            }

            #[inline]
            pub(crate) const fn sub(m: &$T, lhs: $T, rhs: $T) -> $T {
                // this implementation should be equivalent to using overflowing_add and _sub after optimization.
                if lhs >= rhs {
                    lhs - rhs
                } else {
                    *m - (rhs - lhs)
                }
            }

            #[inline]
            pub(crate) const fn neg(m: &$T, target: $T) -> $T {
                match target {
                    0 => 0,
                    x => *m - x
                }
            }
        }
    )*};
}
impl_uprim_vanilla_core_const!(u8 u16 u32 u64 u128 usize);

macro_rules! impl_reduced_binary_pow {
    ($T:ty) => {
        fn pow(&self, base: $T, exp: &$T) -> $T {
            match *exp {
                1 => base,
                2 => self.sqr(base),
                e => {
                    let mut multi = base;
                    let mut exp = e;
                    let mut result = self.transform(1);
                    while exp > 0 {
                        if exp & 1 != 0 {
                            result = self.mul(&result, &multi);
                        }
                        multi = self.sqr(multi);
                        exp >>= 1;
                    }
                    result
                }
            }
        }
    };
}

pub(crate) use impl_reduced_binary_pow;

macro_rules! impl_uprim_vanilla_core {
    ($single:ty) => {
        #[inline(always)]
        fn new(m: &$single) -> Self {
            assert!(m > &0);
            Self(*m)
        }
        #[inline(always)]
        fn transform(&self, target: $single) -> $single {
            target % self.0
        }
        #[inline(always)]
        fn check(&self, target: &$single) -> bool {
            *target < self.0
        }
        #[inline(always)]
        fn residue(&self, target: $single) -> $single {
            target
        }
        #[inline(always)]
        fn modulus(&self) -> $single {
            self.0
        }
        #[inline(always)]
        fn is_zero(&self, target: &$single) -> bool {
            *target == 0
        }

        #[inline(always)]
        fn add(&self, lhs: &$single, rhs: &$single) -> $single {
            Vanilla::<$single>::add(&self.0, *lhs, *rhs)
        }

        #[inline(always)]
        fn dbl(&self, target: $single) -> $single {
            Vanilla::<$single>::dbl(&self.0, target)
        }

        #[inline(always)]
        fn sub(&self, lhs: &$single, rhs: &$single) -> $single {
            Vanilla::<$single>::sub(&self.0, *lhs, *rhs)
        }

        #[inline(always)]
        fn neg(&self, target: $single) -> $single {
            Vanilla::<$single>::neg(&self.0, target)
        }

        #[inline(always)]
        fn inv(&self, target: $single) -> Option<$single> {
            target.invm(&self.0)
        }

        impl_reduced_binary_pow!($single);
    };
}

macro_rules! impl_uprim_vanilla {
    ($t:ident, $ns:ident) => {
        mod $ns {
            use super::*;
            use crate::word::$t::*;

            impl Reducer<$t> for Vanilla<$t> {
                impl_uprim_vanilla_core!($t);

                #[inline]
                fn mul(&self, lhs: &$t, rhs: &$t) -> $t {
                    (wmul(*lhs, *rhs) % extend(self.0)) as $t
                }

                #[inline]
                fn sqr(&self, target: $t) -> $t {
                    (wsqr(target) % extend(self.0)) as $t
                }
            }
        }
    };
}

impl_uprim_vanilla!(u8, u8_impl);
impl_uprim_vanilla!(u16, u16_impl);
impl_uprim_vanilla!(u32, u32_impl);
impl_uprim_vanilla!(u64, u64_impl);
impl_uprim_vanilla!(usize, usize_impl);

impl Reducer<u128> for Vanilla<u128> {
    impl_uprim_vanilla_core!(u128);

    #[inline]
    fn mul(&self, lhs: &u128, rhs: &u128) -> u128 {
        udouble::widening_mul(*lhs, *rhs) % self.0
    }

    #[inline]
    fn sqr(&self, target: u128) -> u128 {
        udouble::widening_square(target) % self.0
    }
}

/// An integer in modulo ring based on conventional [Rem] operations
pub type VanillaInt<T> = ReducedInt<T, Vanilla<T>>;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{ModularCoreOps, ModularPow, ModularUnaryOps};
    use core::marker::PhantomData;
    use rand::random;

    pub(crate) struct ReducedTester<T>(PhantomData<T>);

    macro_rules! impl_reduced_test_for {
        ($($T:ty)*) => {$(
            impl ReducedTester<$T> {
                pub fn test_against_modops<R: Reducer<$T> + Copy>(odd_only: bool) {
                    let mut m = random::<$T>().saturating_add(1);
                    if odd_only {
                        m |= 1;
                    }

                    let (a, b) = (random::<$T>(), random::<$T>());
                    let am = ReducedInt::<$T, R>::new(a, &m);
                    let bm = ReducedInt::<$T, R>::new(b, &m);
                    assert_eq!((am + bm).residue(), a.addm(b, &m), "incorrect add");
                    assert_eq!((am - bm).residue(), a.subm(b, &m), "incorrect sub");
                    assert_eq!((am * bm).residue(), a.mulm(b, &m), "incorrect mul");
                    assert_eq!(am.neg().residue(), a.negm(&m), "incorrect neg");
                    assert_eq!(am.double().residue(), a.dblm(&m), "incorrect dbl");
                    assert_eq!(am.square().residue(), a.sqm(&m), "incorrect sqr");

                    let e = random::<u8>() as $T;
                    assert_eq!(am.pow(&e).residue(), a.powm(e, &m), "incorrect pow");
                    if let Some(v) = a.invm(&m) {
                        assert_eq!(am.inv().unwrap().residue(), v, "incorrect inv");
                    }
                }
            }
        )*};
    }
    impl_reduced_test_for!(u8 u16 u32 u64 u128 usize);

    #[test]
    fn test_against_modops() {
        for _ in 0..10 {
            ReducedTester::<u8>::test_against_modops::<Vanilla<u8>>(false);
            ReducedTester::<u16>::test_against_modops::<Vanilla<u16>>(false);
            ReducedTester::<u32>::test_against_modops::<Vanilla<u32>>(false);
            ReducedTester::<u64>::test_against_modops::<Vanilla<u64>>(false);
            ReducedTester::<u128>::test_against_modops::<Vanilla<u128>>(false);
            ReducedTester::<usize>::test_against_modops::<Vanilla<usize>>(false);
        }
    }
}
