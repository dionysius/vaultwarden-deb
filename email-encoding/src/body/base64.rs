//! Base64 email body encoder.

use core::{
    fmt::{self, Write},
    str,
};

use ::base64::Engine;

const LINE_LEN: usize = 76;
const CRLF: &str = "\r\n";

/// Base64 encode the provided bytes.
///
/// Splits the provided `b` into 57 bytes chunks and
/// base64 encodes them, writing the resulting 76 characters
/// CRLF sequence into `w`.
///
/// The last line may be less than 76 characters in length
/// and will not end in CRLF.
///
/// # Examples
///
/// ```rust
/// # fn main() -> core::fmt::Result {
/// let input = "Hello!
/// You've got mail!
/// This one is base64 encoded.
///
/// Enjoy your bytes 📬📬📬";
///
/// let mut output = String::new();
/// email_encoding::body::base64::encode(input.as_bytes(), &mut output)?;
/// assert_eq!(
///     output,
///     concat!(
///         "SGVsbG8hCllvdSd2ZSBnb3QgbWFpbCEKVGhpcyBvbmUgaXMgYmFzZTY0IGVuY29kZWQuCgpFbmpv\r\n",
///         "eSB5b3VyIGJ5dGVzIPCfk6zwn5Os8J+TrA=="
///     )
/// );
/// # Ok(())
/// # }
/// ```
pub fn encode(b: &[u8], w: &mut dyn Write) -> fmt::Result {
    let mut buf = [0; LINE_LEN];

    let mut chunks = b.chunks(LINE_LEN / 4 * 3).peekable();
    while let Some(chunk) = chunks.next() {
        let len = ::base64::engine::general_purpose::STANDARD
            .encode_slice(chunk, &mut buf)
            .expect("base64 output `buf` is not big enough");

        w.write_str(str::from_utf8(&buf[..len]).expect("base64 produced an invalid encode"))?;
        if chunks.peek().is_some() {
            w.write_str(CRLF)?;
        }
    }

    Ok(())
}

/// Predict how many bytes [`encode`] is going to write given a `input_len` input length.
///
/// # Panics
///
/// Panics if any of the internal calculations overflow.
///
/// # Examples
///
/// ```rust
/// # use email_encoding::body::base64::encoded_len;
/// assert_eq!(encoded_len(0), 0);
/// assert_eq!(encoded_len(16), 24);
/// assert_eq!(encoded_len(300), 410);
/// ```
pub const fn encoded_len(input_len: usize) -> usize {
    // FIXME: use `Option::expect` with MSRV >= 1.83
    macro_rules! checked {
        ($val:expr) => {
            match $val {
                Some(val) => val,
                None => panic!("overflow"),
            }
        };
    }

    let mut base64_len = checked!((input_len / 3).checked_mul(4));
    if input_len % 3 != 0 {
        base64_len = checked!(base64_len.checked_add(4));
    }
    let mut crlf_len = base64_len / LINE_LEN * CRLF.len();
    if crlf_len >= CRLF.len() && base64_len % LINE_LEN == 0 {
        crlf_len -= CRLF.len();
    }
    checked!(base64_len.checked_add(crlf_len))
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use pretty_assertions::assert_eq;

    use super::{encode, encoded_len};

    #[test]
    fn empty() {
        let input = b"";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn oneline() {
        let input = b"012";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "MDEy");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn oneline_padded() {
        let input = b"0123";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(output, "MDEyMw==");
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn multiline() {
        let input =
            b"012345678998765432100123456789987654321001234567899876543210012345678998765432100";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(
            output,
            concat!(
                "MDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2NTQz\r\n",
                "MjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTAw"
            )
        );
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn multiline_padded() {
        let input =
            b"01234567899876543210012345678998765432100123456789987654321001234567899876543210";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(
            output,
            concat!(
                "MDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2NTQz\r\n",
                "MjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTA="
            )
        );
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    fn multiline_exact() {
        let input =
            b"012345678998765432100123456789987654321001234567899876543210012345678998765432100123456789987654321001234567899876543210012345678998765432100123456789987654321001234567899";
        let mut output = String::new();

        encode(input, &mut output).unwrap();

        assert_eq!(
            output,
            concat!(
                "MDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2NTQz\r\n",
                "MjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5ODc2\r\n",
                "NTQzMjEwMDEyMzQ1Njc4OTk4NzY1NDMyMTAwMTIzNDU2Nzg5OTg3NjU0MzIxMDAxMjM0NTY3ODk5"
            )
        );
        assert_eq!(output.len(), encoded_len(input.len()));
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn overflow() {
        encoded_len(usize::MAX);
    }
}
