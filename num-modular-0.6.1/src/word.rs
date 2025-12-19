macro_rules! simple_word_impl {
    ($S:ty, $D:ident) => {
        pub type Word = $S;
        pub type DoubleWord = $D;
        pub use super::$D as DoubleWordModule;

        #[inline(always)]
        pub const fn ones(n: u32) -> Word {
            if n == 0 {
                0
            } else {
                Word::MAX >> (Word::BITS - n)
            }
        }

        #[inline(always)]
        pub const fn extend(word: Word) -> DoubleWord {
            word as DoubleWord
        }

        #[inline(always)]
        pub const fn low(dw: DoubleWord) -> Word {
            dw as Word
        }

        #[inline(always)]
        pub const fn high(dw: DoubleWord) -> Word {
            (dw >> Word::BITS) as Word
        }

        #[inline(always)]
        pub const fn split(dw: DoubleWord) -> (Word, Word) {
            (low(dw), high(dw))
        }

        #[inline(always)]
        pub const fn merge(low: Word, high: Word) -> DoubleWord {
            extend(low) | extend(high) << Word::BITS
        }

        /// Widening multiplication
        #[inline(always)]
        pub const fn wmul(a: Word, b: Word) -> DoubleWord {
            extend(a) * extend(b)
        }

        /// Widening squaring
        #[inline(always)]
        pub const fn wsqr(a: Word) -> DoubleWord {
            extend(a) * extend(a)
        }

        /// Narrowing remainder
        pub const fn nrem(n: DoubleWord, d: Word) -> Word {
            (n % d as DoubleWord) as _
        }
    };
}
use simple_word_impl;

pub mod u8 {
    super::simple_word_impl!(u8, u16);
}

pub mod u16 {
    super::simple_word_impl!(u16, u32);
}

pub mod u32 {
    super::simple_word_impl!(u32, u64);
}

pub mod u64 {
    super::simple_word_impl!(u64, u128);
}

pub mod usize {
    #[cfg(target_pointer_width = "16")]
    super::simple_word_impl!(usize, u32);
    #[cfg(target_pointer_width = "32")]
    super::simple_word_impl!(usize, u64);
    #[cfg(target_pointer_width = "64")]
    super::simple_word_impl!(usize, u128);
}

pub mod u128 {
    use crate::double::udouble;
    pub type Word = u128;
    pub type DoubleWord = udouble;

    #[inline]
    pub const fn extend(word: Word) -> DoubleWord {
        udouble { lo: word, hi: 0 }
    }

    #[inline(always)]
    pub const fn low(dw: DoubleWord) -> Word {
        dw.lo
    }

    #[inline(always)]
    pub const fn high(dw: DoubleWord) -> Word {
        dw.hi
    }

    #[inline]
    pub const fn split(dw: DoubleWord) -> (Word, Word) {
        (dw.lo, dw.hi)
    }

    #[inline]
    pub const fn merge(low: Word, high: Word) -> DoubleWord {
        udouble { lo: low, hi: high }
    }

    #[inline]
    pub const fn wmul(a: Word, b: Word) -> DoubleWord {
        udouble::widening_mul(a, b)
    }

    #[inline]
    pub const fn wsqr(a: Word) -> DoubleWord {
        udouble::widening_square(a)
    }

    #[inline]
    pub fn nrem(n: DoubleWord, d: Word) -> Word {
        n % d
    }
}
