use crate::error::ParseError;

/// An alias to a Result where:
///
/// * `Ok` is `T`.
/// * `Err` is a `ParseError` with context `C` and error `E`
///
/// For a `Result` that is parameterized only by the input type, see
/// [`input::Result`](crate::input::Result).
pub type Result<T, C, E> = std::result::Result<T, ParseError<C, E>>;

#[doc(hidden)]
pub trait IntoResult<T, C, E> {
    fn into_result(self) -> Result<T, C, E>;
}

impl<T, C, E> IntoResult<T, C, E> for T {
    #[inline(always)]
    fn into_result(self) -> Result<T, C, E> {
        Ok(self)
    }
}

impl<T, C, E> IntoResult<T, C, E> for Result<T, C, E> {
    #[inline(always)]
    fn into_result(self) -> Result<T, C, E> {
        self
    }
}

// // This one will result in inference issues when `Ok(T)` is returned.
// impl<T, I: Input, E: ::std::fmt::Display> IntoResult<T, I> for ::std::result::Result<T, E> {
//     fn into_result(self) -> Result<T, I> {
//         let name = unsafe { ::std::intrinsics::type_name::<E>() };
//         self.map_err(|e| ParseError::new(name, e.to_string()))
//     }
// }

// // This one won't but makes some things uglier to write.
// impl<T, I: Input, E2, E1: Into<E2>> IntoResult<T, I, E2> for Result<T, I, E1> {
//     fn into_result(self) -> Result<T, I, E2> {
//         match self {
//             Ok(v) => Ok(v),
//             Err(e) => Err(ParseError {
//                 error: e.error.into(),
//                 contexts: e.contexts
//             })
//         }
//     }
// }

// // This one won't but makes some things uglier to write.
// impl<T, I: Input, E> IntoResult<T, I, B> for Result<T, I, A> {
//     fn into_result(self) -> Result<T, I, B> {
//         self
//     }
// }
