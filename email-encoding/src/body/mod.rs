//! Email body encoding algorithms.

use core::ops::Deref;

pub mod base64;
mod chooser;

/// A possible email `Content-Transfer-Encoding`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// 7bit (US-ASCII)
    SevenBit,
    /// 8bit (UTF-8)
    EightBit,
    /// [Quoted Printable](https://docs.rs/quoted_printable/0.4.5/quoted_printable/fn.encode_to_str.html)
    QuotedPrintable,
    /// [Base64](self::base64::encode)
    Base64,
}

/// A borrowed `str` or `[u8]`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StrOrBytes<'a> {
    /// `str` variant
    Str(&'a str),
    /// `[u8]` variant
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for StrOrBytes<'a> {
    fn from(s: &'a str) -> Self {
        Self::Str(s)
    }
}

impl<'a> From<&'a [u8]> for StrOrBytes<'a> {
    fn from(s: &'a [u8]) -> Self {
        Self::Bytes(s)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for StrOrBytes<'a> {
    fn from(s: &'a [u8; N]) -> Self {
        Self::Bytes(s)
    }
}

impl Deref for StrOrBytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Str(s) => s.as_bytes(),
            Self::Bytes(b) => b,
        }
    }
}
