//! All methods that using pre-computed inverse of the modulus will be contained in this module,
//! as it shares the idea of barret reduction.

// Version 1: Vanilla barret reduction (for x mod n, x < n^2)
// - Choose k = ceil(log2(n))
// - Precompute r = floor(2^(k+1)/n)
// - t = x - floor(x*r/2^(k+1)) * n
// - if t > n, t -= n
// - return t
//
// Version 2: Full width barret reduction
// - Similar to version 1 but support n up to full width
// - Ref (u128): <https://math.stackexchange.com/a/3455956/815652>
//
// Version 3: Floating point barret reduction
// - Using floating point to store r
// - Ref: <http://flintlib.org/doc/ulong_extras.html#c.n_mulmod_precomp>
//
// Version 4: "Improved division by invariant integers" by Granlund
// - Ref: <https://gmplib.org/~tege/division-paper.pdf>
//        <https://gmplib.org/~tege/divcnst-pldi94.pdf>
//
// Comparison between vanilla Barret reduction and Montgomery reduction:
// - Barret reduction requires one 2k-by-k bits and one k-by-k bits multiplication while Montgomery only involves two k-by-k multiplications
// - Extra conversion step is required for Montgomery form to get a normal integer
// (Referece: <https://www.nayuki.io/page/barrett-reduction-algorithm>)
//
// The latter two versions are efficient and practical for use.

use crate::reduced::{impl_reduced_binary_pow, Vanilla};
use crate::{DivExact, ModularUnaryOps, Reducer};

/// Divide a Word by a prearranged divisor.
///
/// Granlund, Montgomerry "Division by Invariant Integers using Multiplication"
/// Algorithm 4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreMulInv1by1<T> {
    // Let n = ceil(log_2(divisor))
    // 2^(n-1) < divisor <= 2^n
    // m = floor(B * 2^n / divisor) + 1 - B, where B = 2^N
    m: T,

    // shift = n - 1
    shift: u32,
}

macro_rules! impl_premulinv_1by1_for {
    ($T:ty) => {
        impl PreMulInv1by1<$T> {
            pub const fn new(divisor: $T) -> Self {
                debug_assert!(divisor > 1);

                // n = ceil(log2(divisor))
                let n = <$T>::BITS - (divisor - 1).leading_zeros();

                /* Calculate:
                 * m = floor(B * 2^n / divisor) + 1 - B
                 * m >= B + 1 - B >= 1
                 * m <= B * 2^n / (2^(n-1) + 1) + 1 - B
                 *    = (B * 2^n + 2^(n-1) + 1) / (2^(n-1) + 1) - B
                 *    = B * (2^n + 2^(n-1-N) + 2^-N) / (2^(n-1)+1) - B
                 *    < B * (2^n + 2^1) / (2^(n-1)+1) - B
                 *    = B
                 * So m fits in a Word.
                 *
                 * Note:
                 * divisor * (B + m) = divisor * floor(B * 2^n / divisor + 1)
                 * = B * 2^n + k, 1 <= k <= divisor
                 */

                // m = floor(B * (2^n-1 - (divisor-1)) / divisor) + 1
                let (lo, _hi) = split(merge(0, ones(n) - (divisor - 1)) / extend(divisor));
                debug_assert!(_hi == 0);
                Self {
                    shift: n - 1,
                    m: lo + 1,
                }
            }

            /// (a / divisor, a % divisor)
            #[inline]
            pub const fn div_rem(&self, a: $T, d: $T) -> ($T, $T) {
                // q = floor( (B + m) * a / (B * 2^n) )
                /*
                 * Remember that divisor * (B + m) = B * 2^n + k, 1 <= k <= 2^n
                 *
                 * (B + m) * a / (B * 2^n)
                 * = a / divisor * (B * 2^n + k) / (B * 2^n)
                 * = a / divisor + k * a / (divisor * B * 2^n)
                 * On one hand, this is >= a / divisor
                 * On the other hand, this is:
                 * <= a / divisor + 2^n * (B-1) / (2^n * B) / divisor
                 * < (a + 1) / divisor
                 *
                 * Therefore the floor is always the exact quotient.
                 */

                // t = m * n / B
                let (_, t) = split(wmul(self.m, a));
                // q = (t + a) / 2^n = (t + (a - t)/2) / 2^(n-1)
                let q = (t + ((a - t) >> 1)) >> self.shift;
                let r = a - q * d;
                (q, r)
            }
        }

        impl DivExact<$T, PreMulInv1by1<$T>> for $T {
            type Output = $T;

            #[inline]
            fn div_exact(self, d: $T, pre: &PreMulInv1by1<$T>) -> Option<Self::Output> {
                let (q, r) = pre.div_rem(self, d);
                if r == 0 {
                    Some(q)
                } else {
                    None
                }
            }
        }
    };
}

/// Divide a DoubleWord by a prearranged divisor.
///
/// Assumes quotient fits in a Word.
///
/// Möller, Granlund, "Improved division by invariant integers", Algorithm 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Normalized2by1Divisor<T> {
    // Normalized (top bit must be set).
    divisor: T,

    // floor((B^2 - 1) / divisor) - B, where B = 2^T::BITS
    m: T,
}

macro_rules! impl_normdiv_2by1_for {
    ($T:ty, $D:ty) => {
        impl Normalized2by1Divisor<$T> {
            /// Calculate the inverse m > 0 of a normalized divisor (fit in a word), such that
            ///
            /// (m + B) * divisor = B^2 - k for some 1 <= k <= divisor
            ///
            #[inline]
            pub const fn invert_word(divisor: $T) -> $T {
                let (m, _hi) = split(<$D>::MAX / extend(divisor));
                debug_assert!(_hi == 1);
                m
            }

            /// Initialize from a given normalized divisor.
            ///
            /// The divisor must have top bit of 1
            #[inline]
            pub const fn new(divisor: $T) -> Self {
                assert!(divisor.leading_zeros() == 0);
                Self {
                    divisor,
                    m: Self::invert_word(divisor),
                }
            }

            /// Returns (a / divisor, a % divisor)
            #[inline]
            pub const fn div_rem_1by1(&self, a: $T) -> ($T, $T) {
                if a < self.divisor {
                    (0, a)
                } else {
                    (1, a - self.divisor) // because self.divisor is normalized
                }
            }

            /// Returns (a / divisor, a % divisor)
            /// The result must fit in a single word.
            #[inline]
            pub const fn div_rem_2by1(&self, a: $D) -> ($T, $T) {
                let (a_lo, a_hi) = split(a);
                debug_assert!(a_hi < self.divisor);

                // Approximate quotient is (m + B) * a / B^2 ~= (m * a/B + a)/B.
                // This is q1 below.
                // This doesn't overflow because a_hi < self.divisor <= Word::MAX.
                let (q0, q1) = split(wmul(self.m, a_hi) + a);

                // q = q1 + 1 is our first approximation, but calculate mod B.
                // r = a - q * d
                let q = q1.wrapping_add(1);
                let r = a_lo.wrapping_sub(q.wrapping_mul(self.divisor));

                /* Theorem: max(-d, q0+1-B) <= r < max(B-d, q0)
                 * Proof:
                 * r = a - q * d = a - q1 * d - d
                 * = a - (q1 * B + q0 - q0) * d/B - d
                 * = a - (m * a_hi + a - q0) * d/B - d
                 * = a - ((m+B) * a_hi + a_lo - q0) * d/B - d
                 * = a - ((B^2-k)/d * a_hi + a_lo - q0) * d/B - d
                 * = a - B * a_hi + (a_hi * k - a_lo * d + q0 * d) / B - d
                 * = (a_hi * k + a_lo * (B - d) + q0 * d) / B - d
                 *
                 * r >= q0 * d / B - d
                 * r >= -d
                 * r >= d/B (q0 - B) > q0-B
                 * r >= max(-d, q0+1-B)
                 *
                 * r < (d * d + B * (B-d) + q0 * d) / B - d
                 * = (B-d)^2 / B + q0 * d / B
                 * = (1 - d/B) * (B-d) + (d/B) * q0
                 * <= max(B-d, q0)
                 * QED
                 */

                // if r mod B > q0 { q -= 1; r += d; }
                //
                // Consider two cases:
                // a) r >= 0:
                // Then r = r mod B > q0, hence r < B-d. Adding d will not overflow r.
                // b) r < 0:
                // Then r mod B = r-B > q0, and r >= -d, so adding d will make r non-negative.
                // In either case, this will result in 0 <= r < B.

                // In a branch-free way:
                // decrease = 0xffff.fff = -1 if r mod B > q0, 0 otherwise.
                let (_, decrease) = split(extend(q0).wrapping_sub(extend(r)));
                let mut q = q.wrapping_add(decrease);
                let mut r = r.wrapping_add(decrease & self.divisor);

                // At this point 0 <= r < B, i.e. 0 <= r < 2d.
                // the following fix step is unlikely to happen
                if r >= self.divisor {
                    q += 1;
                    r -= self.divisor;
                }

                (q, r)
            }
        }
    };
}

/// A wrapper of [Normalized2by1Divisor] that can be used as a [Reducer]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreMulInv2by1<T> {
    div: Normalized2by1Divisor<T>,
    shift: u32,
}

impl<T> PreMulInv2by1<T> {
    #[inline]
    pub const fn divider(&self) -> &Normalized2by1Divisor<T> {
        &self.div
    }
    #[inline]
    pub const fn shift(&self) -> u32 {
        self.shift
    }
}

macro_rules! impl_premulinv_2by1_reducer_for {
    ($T:ty) => {
        impl PreMulInv2by1<$T> {
            #[inline]
            pub const fn new(divisor: $T) -> Self {
                let shift = divisor.leading_zeros();
                let div = Normalized2by1Divisor::<$T>::new(divisor << shift);
                Self { div, shift }
            }

            /// Get the **normalized** divisor.
            #[inline]
            pub const fn divisor(&self) -> $T {
                self.div.divisor
            }
        }

        impl Reducer<$T> for PreMulInv2by1<$T> {
            #[inline]
            fn new(m: &$T) -> Self {
                PreMulInv2by1::<$T>::new(*m)
            }
            #[inline]
            fn transform(&self, target: $T) -> $T {
                if self.shift == 0 {
                    self.div.div_rem_1by1(target).1
                } else {
                    self.div.div_rem_2by1(extend(target) << self.shift).1
                }
            }
            #[inline]
            fn check(&self, target: &$T) -> bool {
                *target < self.div.divisor && target & ones(self.shift) == 0
            }
            #[inline]
            fn residue(&self, target: $T) -> $T {
                target >> self.shift
            }
            #[inline]
            fn modulus(&self) -> $T {
                self.div.divisor >> self.shift
            }
            #[inline]
            fn is_zero(&self, target: &$T) -> bool {
                *target == 0
            }

            #[inline(always)]
            fn add(&self, lhs: &$T, rhs: &$T) -> $T {
                Vanilla::<$T>::add(&self.div.divisor, *lhs, *rhs)
            }
            #[inline(always)]
            fn dbl(&self, target: $T) -> $T {
                Vanilla::<$T>::dbl(&self.div.divisor, target)
            }
            #[inline(always)]
            fn sub(&self, lhs: &$T, rhs: &$T) -> $T {
                Vanilla::<$T>::sub(&self.div.divisor, *lhs, *rhs)
            }
            #[inline(always)]
            fn neg(&self, target: $T) -> $T {
                Vanilla::<$T>::neg(&self.div.divisor, target)
            }

            #[inline(always)]
            fn inv(&self, target: $T) -> Option<$T> {
                self.residue(target)
                    .invm(&self.modulus())
                    .map(|v| v << self.shift)
            }
            #[inline]
            fn mul(&self, lhs: &$T, rhs: &$T) -> $T {
                self.div.div_rem_2by1(wmul(lhs >> self.shift, *rhs)).1
            }
            #[inline]
            fn sqr(&self, target: $T) -> $T {
                self.div.div_rem_2by1(wsqr(target) >> self.shift).1
            }

            impl_reduced_binary_pow!($T);
        }
    };
}

/// Divide a 3-Word by a prearranged DoubleWord divisor.
///
/// Assumes quotient fits in a Word.
///
/// Möller, Granlund, "Improved division by invariant integers"
/// Algorithm 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Normalized3by2Divisor<T, D> {
    // Top bit must be 1.
    divisor: D,

    // floor ((B^3 - 1) / divisor) - B, where B = 2^WORD_BITS
    m: T,
}

macro_rules! impl_normdiv_3by2_for {
    ($T:ty, $D:ty) => {
        impl Normalized3by2Divisor<$T, $D> {
            /// Calculate the inverse m > 0 of a normalized divisor (fit in a DoubleWord), such that
            ///
            /// (m + B) * divisor = B^3 - k for some 1 <= k <= divisor
            ///
            /// Möller, Granlund, "Improved division by invariant integers", Algorithm 6.
            #[inline]
            pub const fn invert_double_word(divisor: $D) -> $T {
                let (d0, d1) = split(divisor);
                let mut v = Normalized2by1Divisor::<$T>::invert_word(d1);
                // then B^2 - d1 <= (B + v)d1 < B^2

                let (mut p, c) = d1.wrapping_mul(v).overflowing_add(d0);
                if c {
                    v -= 1;
                    if p >= d1 {
                        v -= 1;
                        p -= d1;
                    }
                    p = p.wrapping_sub(d1);
                }
                // then B^2 - d1 <= (B + v)d1 + d0 < B^2

                let (t0, t1) = split(extend(v) * extend(d0));
                let (p, c) = p.overflowing_add(t1);
                if c {
                    v -= 1;
                    if merge(t0, p) >= divisor {
                        v -= 1;
                    }
                }

                v
            }

            /// Initialize from a given normalized divisor.
            ///
            /// divisor must have top bit of 1
            #[inline]
            pub const fn new(divisor: $D) -> Self {
                assert!(divisor.leading_zeros() == 0);
                Self {
                    divisor,
                    m: Self::invert_double_word(divisor),
                }
            }

            #[inline]
            pub const fn div_rem_2by2(&self, a: $D) -> ($D, $D) {
                if a < self.divisor {
                    (0, a)
                } else {
                    (1, a - self.divisor) // because self.divisor is normalized
                }
            }

            /// The input a is arranged as (lo, mi & hi)
            /// The output is (a / divisor, a % divisor)
            pub const fn div_rem_3by2(&self, a_lo: $T, a_hi: $D) -> ($T, $D) {
                debug_assert!(a_hi < self.divisor);
                let (a1, a2) = split(a_hi);
                let (d0, d1) = split(self.divisor);

                // This doesn't overflow because a2 <= self.divisor / B <= Word::MAX.
                let (q0, q1) = split(wmul(self.m, a2) + a_hi);
                let r1 = a1.wrapping_sub(q1.wrapping_mul(d1));
                let t = wmul(d0, q1);
                let r = merge(a_lo, r1).wrapping_sub(t).wrapping_sub(self.divisor);

                // The first guess of quotient is q1 + 1
                // if r1 >= q0 { r += d; } else { q1 += 1; }
                // In a branch-free way:
                // decrease = 0 if r1 >= q0, = 0xffff.fff = -1 otherwise
                let (_, r1) = split(r);
                let (_, decrease) = split(extend(r1).wrapping_sub(extend(q0)));
                let mut q1 = q1.wrapping_sub(decrease);
                let mut r = r.wrapping_add(merge(!decrease, !decrease) & self.divisor);

                // the following fix step is unlikely to happen
                if r >= self.divisor {
                    q1 += 1;
                    r -= self.divisor;
                }

                (q1, r)
            }

            /// Divdide a 4-word number with double word divisor
            ///
            /// The output is (a / divisor, a % divisor)
            pub const fn div_rem_4by2(&self, a_lo: $D, a_hi: $D) -> ($D, $D) {
                let (a0, a1) = split(a_lo);
                let (q1, r1) = self.div_rem_3by2(a1, a_hi);
                let (q0, r0) = self.div_rem_3by2(a0, r1);
                (merge(q0, q1), r0)
            }
        }
    };
}

/// A wrapper of [Normalized3by2Divisor] that can be used as a [Reducer]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreMulInv3by2<T, D> {
    div: Normalized3by2Divisor<T, D>,
    shift: u32,
}

impl<T, D> PreMulInv3by2<T, D> {
    #[inline]
    pub const fn divider(&self) -> &Normalized3by2Divisor<T, D> {
        &self.div
    }
    #[inline]
    pub const fn shift(&self) -> u32 {
        self.shift
    }
}

macro_rules! impl_premulinv_3by2_reducer_for {
    ($T:ty, $D:ty) => {
        impl PreMulInv3by2<$T, $D> {
            #[inline]
            pub const fn new(divisor: $D) -> Self {
                let shift = divisor.leading_zeros();
                let div = Normalized3by2Divisor::<$T, $D>::new(divisor << shift);
                Self { div, shift }
            }

            /// Get the **normalized** divisor.
            #[inline]
            pub const fn divisor(&self) -> $D {
                self.div.divisor
            }
        }

        impl Reducer<$D> for PreMulInv3by2<$T, $D> {
            #[inline]
            fn new(m: &$D) -> Self {
                assert!(*m > <$T>::MAX as $D);
                let shift = m.leading_zeros();
                let div = Normalized3by2Divisor::<$T, $D>::new(m << shift);
                Self { div, shift }
            }
            #[inline]
            fn transform(&self, target: $D) -> $D {
                if self.shift == 0 {
                    self.div.div_rem_2by2(target).1
                } else {
                    let (lo, hi) = split(target);
                    let (n0, carry) = split(extend(lo) << self.shift);
                    let n12 = (extend(hi) << self.shift) | extend(carry);
                    self.div.div_rem_3by2(n0, n12).1
                }
            }
            #[inline]
            fn check(&self, target: &$D) -> bool {
                *target < self.div.divisor && split(*target).0 & ones(self.shift) == 0
            }
            #[inline]
            fn residue(&self, target: $D) -> $D {
                target >> self.shift
            }
            #[inline]
            fn modulus(&self) -> $D {
                self.div.divisor >> self.shift
            }
            #[inline]
            fn is_zero(&self, target: &$D) -> bool {
                *target == 0
            }

            #[inline(always)]
            fn add(&self, lhs: &$D, rhs: &$D) -> $D {
                Vanilla::<$D>::add(&self.div.divisor, *lhs, *rhs)
            }
            #[inline(always)]
            fn dbl(&self, target: $D) -> $D {
                Vanilla::<$D>::dbl(&self.div.divisor, target)
            }
            #[inline(always)]
            fn sub(&self, lhs: &$D, rhs: &$D) -> $D {
                Vanilla::<$D>::sub(&self.div.divisor, *lhs, *rhs)
            }
            #[inline(always)]
            fn neg(&self, target: $D) -> $D {
                Vanilla::<$D>::neg(&self.div.divisor, target)
            }

            #[inline(always)]
            fn inv(&self, target: $D) -> Option<$D> {
                self.residue(target)
                    .invm(&self.modulus())
                    .map(|v| v << self.shift)
            }
            #[inline]
            fn mul(&self, lhs: &$D, rhs: &$D) -> $D {
                let prod = DoubleWordModule::wmul(lhs >> self.shift, *rhs);
                let (lo, hi) = DoubleWordModule::split(prod);
                self.div.div_rem_4by2(lo, hi).1
            }
            #[inline]
            fn sqr(&self, target: $D) -> $D {
                let prod = DoubleWordModule::wsqr(target) >> self.shift;
                let (lo, hi) = DoubleWordModule::split(prod);
                self.div.div_rem_4by2(lo, hi).1
            }

            impl_reduced_binary_pow!($D);
        }
    };
}

macro_rules! collect_impls {
    ($T:ident, $ns:ident) => {
        mod $ns {
            use super::*;
            use crate::word::$T::*;

            impl_premulinv_1by1_for!(Word);
            impl_normdiv_2by1_for!(Word, DoubleWord);
            impl_premulinv_2by1_reducer_for!(Word);
            impl_normdiv_3by2_for!(Word, DoubleWord);
            impl_premulinv_3by2_reducer_for!(Word, DoubleWord);
        }
    };
}
collect_impls!(u8, u8_impl);
collect_impls!(u16, u16_impl);
collect_impls!(u32, u32_impl);
collect_impls!(u64, u64_impl);
collect_impls!(usize, usize_impl);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduced::tests::ReducedTester;
    use rand::prelude::*;

    #[test]
    fn test_mul_inv_1by1() {
        type Word = u64;
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..400000 {
            let d_bits = rng.gen_range(2..=Word::BITS);
            let max_d = Word::MAX >> (Word::BITS - d_bits);
            let d = rng.gen_range(max_d / 2 + 1..=max_d);
            let fast_div = PreMulInv1by1::<Word>::new(d);
            let n = rng.gen();
            let (q, r) = fast_div.div_rem(n, d);
            assert_eq!(q, n / d);
            assert_eq!(r, n % d);

            if r == 0 {
                assert_eq!(n.div_exact(d, &fast_div), Some(q));
            } else {
                assert_eq!(n.div_exact(d, &fast_div), None);
            }
        }
    }

    #[test]
    fn test_mul_inv_2by1() {
        type Word = u64;
        type Divider = Normalized2by1Divisor<Word>;
        use crate::word::u64::*;

        let fast_div = Divider::new(Word::MAX);
        assert_eq!(fast_div.div_rem_2by1(0), (0, 0));

        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..200000 {
            let d = rng.gen_range(Word::MAX / 2 + 1..=Word::MAX);
            let q = rng.gen();
            let r = rng.gen_range(0..d);
            let (a0, a1) = split(wmul(q, d) + extend(r));
            let fast_div = Divider::new(d);
            assert_eq!(fast_div.div_rem_2by1(merge(a0, a1)), (q, r));
        }
    }

    #[test]
    fn test_mul_inv_3by2() {
        type Word = u64;
        type DoubleWord = u128;
        type Divider = Normalized3by2Divisor<Word, DoubleWord>;
        use crate::word::u64::*;

        let d = DoubleWord::MAX;
        let fast_div = Divider::new(d);
        assert_eq!(fast_div.div_rem_3by2(0, 0), (0, 0));

        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..100000 {
            let d = rng.gen_range(DoubleWord::MAX / 2 + 1..=DoubleWord::MAX);
            let r = rng.gen_range(0..d);
            let q = rng.gen();

            let (d0, d1) = split(d);
            let (r0, r1) = split(r);
            let (a0, c) = split(wmul(q, d0) + extend(r0));
            let (a1, a2) = split(wmul(q, d1) + extend(r1) + extend(c));
            let a12 = merge(a1, a2);

            let fast_div = Divider::new(d);
            assert_eq!(
                fast_div.div_rem_3by2(a0, a12),
                (q, r),
                "failed at {:?} / {}",
                (a0, a12),
                d
            );
        }
    }

    #[test]
    fn test_mul_inv_4by2() {
        type Word = u64;
        type DoubleWord = u128;
        type Divider = Normalized3by2Divisor<Word, DoubleWord>;
        use crate::word::u128::*;

        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..20000 {
            let d = rng.gen_range(DoubleWord::MAX / 2 + 1..=DoubleWord::MAX);
            let q = rng.gen();
            let r = rng.gen_range(0..d);
            let (a_lo, a_hi) = split(wmul(q, d) + r as DoubleWord);
            let fast_div = Divider::new(d);
            assert_eq!(fast_div.div_rem_4by2(a_lo, a_hi), (q, r));
        }
    }

    #[test]
    fn test_2by1_against_modops() {
        for _ in 0..10 {
            ReducedTester::<u8>::test_against_modops::<PreMulInv2by1<u8>>(false);
            ReducedTester::<u16>::test_against_modops::<PreMulInv2by1<u16>>(false);
            ReducedTester::<u32>::test_against_modops::<PreMulInv2by1<u32>>(false);
            ReducedTester::<u64>::test_against_modops::<PreMulInv2by1<u64>>(false);
            // ReducedTester::<u128>::test_against_modops::<PreMulInv2by1<u128>>();
            ReducedTester::<usize>::test_against_modops::<PreMulInv2by1<usize>>(false);
        }
    }

    #[test]
    fn test_3by2_against_modops() {
        for _ in 0..10 {
            ReducedTester::<u16>::test_against_modops::<PreMulInv3by2<u8, u16>>(false);
            ReducedTester::<u32>::test_against_modops::<PreMulInv3by2<u16, u32>>(false);
            ReducedTester::<u64>::test_against_modops::<PreMulInv3by2<u32, u64>>(false);
            ReducedTester::<u128>::test_against_modops::<PreMulInv3by2<u64, u128>>(false);
        }
    }
}
