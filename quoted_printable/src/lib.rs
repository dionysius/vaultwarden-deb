#![forbid(unsafe_code)]
#![no_std]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod lib {
    #[cfg(not(feature = "std"))]
    pub use alloc::fmt;
    #[cfg(feature = "std")]
    pub use std::fmt;

    #[cfg(not(feature = "std"))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    #[cfg(not(feature = "std"))]
    pub use alloc::string::String;
    #[cfg(feature = "std")]
    pub use std::string::String;
}

static HEX_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

/// A flag that allows control over the decoding strictness.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseMode {
    /// Perform strict checking over the input, and return an error if any
    /// input appears malformed.
    Strict,
    /// Perform robust parsing, and gracefully handle any malformed input. This
    /// can result in the decoded output being different than what was intended.
    Robust,
}

/// A flag that controls how to treat the input when encoding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputMode {
    /// Treat the input as text, and don't encode CRLF pairs.
    Text,
    /// Treat the input as binary, and encode all CRLF pairs.
    Binary,
}

/// Options to control encoding and decoding behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    /// Line length at which to wrap when encoding. Also
    /// determines if input is valid or not when decoding in
    /// strict mode.
    line_length_limit: usize,
    /// How to treat the input when encoding.
    input_mode: InputMode,
    /// How strict to be while decoding.
    parse_mode: ParseMode,
}

impl Options {
    pub fn default() -> Self {
        Options {
            line_length_limit: 76,
            input_mode: InputMode::Text,
            parse_mode: ParseMode::Robust,
        }
    }

    pub fn line_length_limit(mut self, limit: usize) -> Self {
        self.line_length_limit = limit;
        self
    }

    pub fn input_mode(mut self, mode: InputMode) -> Self {
        self.input_mode = mode;
        self
    }

    pub fn parse_mode(mut self, mode: ParseMode) -> Self {
        self.parse_mode = mode;
        self
    }
}

/// An error type that represents different kinds of decoding errors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuotedPrintableError {
    /// A byte was found in the input that was outside of the allowed range. The
    /// allowed range is the horizontal tab (ASCII 0x09), CR/LF characters (ASCII
    /// 0x0D and 0x0A), and anything in the ASCII range 0x20 to 0x7E, inclusive.
    InvalidByte,
    /// Lines where found in the input that exceeded 76 bytes in length, excluding
    /// the terminating CRLF.
    LineTooLong,
    /// An '=' character was found in the input without the proper number of
    /// hex-characters following it. This includes '=' characters followed
    /// by a single character and then the CRLF pair, for example.
    IncompleteHexOctet,
    /// An '=' character was found with two following characters, but they were
    /// not hex characters. '=Hi' for example would be an invalid encoding.
    InvalidHexOctet,
    /// An '=' character was found with two following hex characters, but the
    /// hex characters were lowercase rather than uppercase. The spec explicitly
    /// requires uppercase hex to be used, so this is considered an error.
    LowercaseHexOctet,
}

impl lib::fmt::Display for QuotedPrintableError {
    fn fmt(&self, f: &mut lib::fmt::Formatter) -> lib::fmt::Result {
        match *self {
            QuotedPrintableError::InvalidByte => {
                write!(
                    f,
                    "A unallowed byte was found in the quoted-printable input"
                )
            }
            QuotedPrintableError::LineTooLong => {
                write!(
                    f,
                    "A line length in the quoted-printed input exceeded 76 bytes"
                )
            }
            QuotedPrintableError::IncompleteHexOctet => {
                write!(
                    f,
                    "A '=' followed by only one character was found in the input"
                )
            }
            QuotedPrintableError::InvalidHexOctet => {
                write!(
                    f,
                    "A '=' followed by non-hex characters was found in the input"
                )
            }
            QuotedPrintableError::LowercaseHexOctet => {
                write!(f, "A '=' was followed by lowercase hex characters")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for QuotedPrintableError {
    fn description(&self) -> &str {
        "invalid quoted-printable input"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

/// Decodes a piece of quoted-printable data.
/// This implementation is equivalent to `decode_with_options` with an
/// `Options` that uses the provided `ParseMode`.
#[inline(always)]
pub fn decode<R: AsRef<[u8]>>(
    input: R,
    mode: ParseMode,
) -> Result<lib::Vec<u8>, QuotedPrintableError> {
    _decode(input.as_ref(), Options::default().parse_mode(mode))
}

/// Decodes a piece of quoted-printable data.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function attempts to decode input that is conformant with that
/// spec. Note that quoted-printable encoding is independent of charset, and so
/// this function returns a Vec<u8> of bytes upon success. It is up to the caller
/// to convert that to a String if desired; the charset required to do so must
/// come from somewhere else.
///
/// # Examples
///
/// ```
///     use quoted_printable::{decode, ParseMode};
///     let decoded = decode("hello=3Dworld=0D=0A".as_bytes(), ParseMode::Robust).unwrap();
///     assert_eq!("hello=world\r\n", String::from_utf8(decoded).unwrap());
/// ```
///
/// # Errors
///
/// If this function is called with the ParseMode::Strict option, then it may return
/// a QuotedPrintableError if it detects that the input does not strictly conform
/// to the quoted-printable spec. If this function is called with ParseMode::Robust,
/// then it will attempt to gracefully handle any errors that arise. This might
/// result in input bytes being stripped out and ignored in some cases. Refer
/// to IETF RFC 2045, section 6.7 for details on what constitutes valid and
/// invalid input, and what a "robust" implementation would do in the face of
/// invalid input.
#[inline(always)]
pub fn decode_with_options<R: AsRef<[u8]>>(
    input: R,
    options: Options,
) -> Result<lib::Vec<u8>, QuotedPrintableError> {
    _decode(input.as_ref(), options)
}

fn _decode(input: &[u8], options: Options) -> Result<lib::Vec<u8>, QuotedPrintableError> {
    let filtered = input
        .into_iter()
        .filter_map(|&c| match c {
            b'\t' | b'\r' | b'\n' | b' '..=b'~' => Some(c as char),
            _ => None,
        })
        .collect::<lib::String>();
    if options.parse_mode == ParseMode::Strict && filtered.len() != input.len() {
        return Err(QuotedPrintableError::InvalidByte);
    }
    let mut decoded = lib::Vec::new();
    let mut lines = filtered.lines();
    let mut add_line_break = None;
    loop {
        let mut bytes = match lines.next() {
            Some(v) => v.trim_end().bytes(),
            None => {
                if options.parse_mode == ParseMode::Strict && add_line_break == Some(false) {
                    return Err(QuotedPrintableError::IncompleteHexOctet);
                }
                break;
            }
        };

        if options.parse_mode == ParseMode::Strict && bytes.len() > options.line_length_limit {
            return Err(QuotedPrintableError::LineTooLong);
        }

        if add_line_break == Some(true) {
            decoded.push(b'\r');
            decoded.push(b'\n');
            add_line_break = Some(false);
        }

        loop {
            let byte = match bytes.next() {
                Some(v) => v,
                None => {
                    add_line_break = Some(true);
                    break;
                }
            };

            if byte == b'=' {
                let upper = match bytes.next() {
                    Some(v) => v,
                    None => break,
                };
                let lower = match bytes.next() {
                    Some(v) => v,
                    None => {
                        if options.parse_mode == ParseMode::Strict {
                            return Err(QuotedPrintableError::IncompleteHexOctet);
                        }
                        decoded.push(byte);
                        decoded.push(upper);
                        add_line_break = Some(true);
                        break;
                    }
                };
                let upper_char = upper as char;
                let lower_char = lower as char;
                if upper_char.is_digit(16) && lower_char.is_digit(16) {
                    if options.parse_mode == ParseMode::Strict {
                        if upper_char.to_uppercase().next() != Some(upper_char)
                            || lower_char.to_uppercase().next() != Some(lower_char)
                        {
                            return Err(QuotedPrintableError::LowercaseHexOctet);
                        }
                    }
                    let combined =
                        upper_char.to_digit(16).unwrap() << 4 | lower_char.to_digit(16).unwrap();
                    decoded.push(combined as u8);
                } else {
                    if options.parse_mode == ParseMode::Strict {
                        return Err(QuotedPrintableError::InvalidHexOctet);
                    }
                    decoded.push(byte);
                    decoded.push(upper);
                    decoded.push(lower);
                }
            } else {
                decoded.push(byte);
            }
        }
    }

    if filtered.ends_with('\n') {
        // the filtered.lines() call above ignores trailing newlines instead
        // of returning an empty string in the last element. So if there was
        // a trailing newline, let's tack on the CRLF to carry that through
        // the decoder.
        decoded.push(b'\r');
        decoded.push(b'\n');
    }

    Ok(decoded)
}

fn append(
    result: &mut lib::String,
    to_append: &[char],
    bytes_on_line: &mut usize,
    limit: usize,
    backup_pos: &mut usize,
) {
    if *bytes_on_line + to_append.len() > limit {
        if *bytes_on_line == limit {
            // We're already at the max length, so inserting the '=' in the soft
            // line break would put us over. Instead, we insert the soft line
            // break at the backup pos, which is just before the last thing
            // appended.
            *bytes_on_line = result.len() - *backup_pos;
            result.insert_str(*backup_pos, "=\r\n");
        } else {
            result.push_str("=\r\n");
            *bytes_on_line = 0;
        }
    }
    result.extend(to_append);
    *bytes_on_line = *bytes_on_line + to_append.len();
    *backup_pos = result.len() - to_append.len();
}

fn encode_trailing_space_tab(
    result: &mut lib::String,
    bytes_on_line: &mut usize,
    limit: usize,
    backup_pos: &mut usize,
) {
    // If the last character before a CRLF was a space or tab, then encode it
    // since "Octets with values of 9 and 32 ... MUST NOT be so represented
    // at the end of an encoded line." We can just pop it off the end of the
    // result and append the encoded version. The encoded version may end up
    // getting bumped to a new line, but in that case we know that the soft
    // line break '=' will always fit because we're removing one char before
    // calling append.
    match result.chars().last() {
        Some(' ') => {
            *bytes_on_line -= 1;
            result.pop();
            append(result, &['=', '2', '0'], bytes_on_line, limit, backup_pos);
        }
        Some('\t') => {
            *bytes_on_line -= 1;
            result.pop();
            append(result, &['=', '0', '9'], bytes_on_line, limit, backup_pos);
        }
        _ => (),
    };
}

/// Encodes some bytes into quoted-printable format, treating the input as text.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function encodes a set of raw bytes into a format conformant with
/// that spec. The output contains CRLF pairs as needed so that each line is
/// wrapped to 76 characters or less (not including the CRLF).
///
/// # Examples
///
/// ```
///     use quoted_printable::encode;
///     let encoded = encode("hello, \u{20ac} zone!");
///     assert_eq!("hello, =E2=82=AC zone!", String::from_utf8(encoded).unwrap());
/// ```
#[inline(always)]
pub fn encode<R: AsRef<[u8]>>(input: R) -> lib::Vec<u8> {
    let encoded_as_string = _encode(
        input.as_ref(),
        Options::default().input_mode(InputMode::Text),
    );
    encoded_as_string.into()
}

/// Encodes some bytes into quoted-printable format, treating the input as binary.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function encodes a set of raw bytes into a format conformant with
/// that spec. The output contains CRLF pairs as needed so that each line is
/// wrapped to 76 characters or less (not including the CRLF).
///
/// # Examples
///
/// ```
///     use quoted_printable::encode_binary;
///     let encoded = encode_binary("hello, \u{20ac} zone!\r\n");
///     assert_eq!("hello, =E2=82=AC zone!=0D=0A", String::from_utf8(encoded).unwrap());
/// ```
#[inline(always)]
pub fn encode_binary<R: AsRef<[u8]>>(input: R) -> lib::Vec<u8> {
    let encoded_as_string = _encode(
        input.as_ref(),
        Options::default().input_mode(InputMode::Binary),
    );
    encoded_as_string.into()
}

fn _encode(input: &[u8], options: Options) -> lib::String {
    let limit = options.line_length_limit;
    let mut result = lib::String::new();
    let mut on_line: usize = 0;
    let mut backup_pos: usize = 0;
    let mut was_cr = false;

    let mut it = input.iter().peekable();
    while let Some(&byte) = it.next() {
        if was_cr {
            if byte == b'\n' {
                encode_trailing_space_tab(&mut result, &mut on_line, limit, &mut backup_pos);
                match options.input_mode {
                    InputMode::Text => {
                        result.push_str("\r\n");
                        on_line = 0;
                    }
                    InputMode::Binary => {
                        append(
                            &mut result,
                            &['=', '0', 'D'],
                            &mut on_line,
                            limit,
                            &mut backup_pos,
                        );
                        append(
                            &mut result,
                            &['=', '0', 'A'],
                            &mut on_line,
                            limit,
                            &mut backup_pos,
                        );
                    }
                };
                was_cr = false;
                continue;
            }
            // encode the CR ('\r') we skipped over before
            append(
                &mut result,
                &['=', '0', 'D'],
                &mut on_line,
                limit,
                &mut backup_pos,
            );
        }
        if byte == b'\r' {
            // remember we had a CR ('\r') but do not encode it yet
            was_cr = true;
            continue;
        } else {
            was_cr = false;
        }

        // look for runs of characters that don't need encoding - this
        // is very common (QP is normally used on ASCII text where
        // most characters don't need encoding) and much faster than
        // calling encode_byte on each character.  To keep this from
        // completely reimplementing append(), only do this if we have
        // at least 3 characters left on the line and don't try to
        // deal with the line-ending stuff.
        if limit - on_line >= 3 && !needs_encoding(byte) {
            // peek ahead up to max line length and copy the run directly into the output
            let mut run_len: usize = 1;
            let max_run_len: usize = limit - on_line - 2;
            debug_assert!(max_run_len >= run_len);

            // add the char to result directly - safe because we know we're not at the line length limit
            result.push(byte as char);

            // look ahead for a run of characters we can put directly into the result
            while let Some(&&next_byte) = it.peek() {
                if run_len == max_run_len {
                    break;
                }
                if needs_encoding(next_byte) {
                    break;
                }

                run_len += 1;

                // add the next char to result directly - this is safe
                // because we're not close to the line length limit
                result.push(next_byte as char);

                // consume the byte so we don't see it again
                it.next();
            }

            // update counters for where we are in the line and what was last appended
            on_line += run_len;
            backup_pos = result.len();

            continue;
        }

        encode_byte(&mut result, byte, &mut on_line, limit, &mut backup_pos);
    }

    // we haven't yet encoded the last CR ('\r') so do it now
    if was_cr {
        append(
            &mut result,
            &['=', '0', 'D'],
            &mut on_line,
            limit,
            &mut backup_pos,
        );
    } else {
        encode_trailing_space_tab(&mut result, &mut on_line, limit, &mut backup_pos);
    }

    result
}

#[inline(always)]
fn needs_encoding(c: u8) -> bool {
    return match c {
        b'=' => true,
        b'\t' | b' '..=b'~' => false,
        _ => true,
    };
}

/// Encodes some bytes into quoted-printable format.
///
/// The difference to `encode` is that this function returns a `String`.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function encodes a set of raw bytes into a format conformant with
/// that spec. The output contains CRLF pairs as needed so that each line is
/// wrapped to 76 characters or less (not including the CRLF).
///
/// # Examples
///
/// ```
///     use quoted_printable::encode_to_str;
///     let encoded = encode_to_str("hello, \u{20ac} zone!");
///     assert_eq!("hello, =E2=82=AC zone!", encoded);
/// ```
#[inline(always)]
pub fn encode_to_str<R: AsRef<[u8]>>(input: R) -> lib::String {
    _encode(
        input.as_ref(),
        Options::default().input_mode(InputMode::Text),
    )
}

/// Encodes some bytes into quoted-printable format.
///
/// The difference to `encode_binary` is that this function returns a `String`.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function encodes a set of raw bytes into a format conformant with
/// that spec. The output contains CRLF pairs as needed so that each line is
/// wrapped to 76 characters or less (not including the CRLF).
///
/// # Examples
///
/// ```
///     use quoted_printable::encode_binary_to_str;
///     let encoded = encode_binary_to_str("hello, \u{20ac} zone!\r\n");
///     assert_eq!("hello, =E2=82=AC zone!=0D=0A", encoded);
/// ```
#[inline(always)]
pub fn encode_binary_to_str<R: AsRef<[u8]>>(input: R) -> lib::String {
    _encode(
        input.as_ref(),
        Options::default().input_mode(InputMode::Binary),
    )
}

/// Encodes some bytes into quoted-printable format, using the provided options.
///
/// The quoted-printable transfer-encoding is defined in IETF RFC 2045, section
/// 6.7. This function encodes a set of raw bytes into a format conformant with
/// that spec. The output contains CRLF pairs as needed so that each line is
/// wrapped to 76 characters or less (not including the CRLF).
///
/// # Examples
///
/// ```
///     use quoted_printable::{encode_with_options, Options};
///     let encoded = encode_with_options("hello, \u{20ac} zone!", Options::default());
///     assert_eq!("hello, =E2=82=AC zone!", encoded);
/// ```
#[inline(always)]
pub fn encode_with_options<R: AsRef<[u8]>>(input: R, options: Options) -> lib::String {
    _encode(input.as_ref(), options)
}

#[inline]
fn encode_byte(
    result: &mut lib::String,
    to_append: u8,
    on_line: &mut usize,
    limit: usize,
    backup_pos: &mut usize,
) {
    match to_append {
        b'=' => append(result, &['=', '3', 'D'], on_line, limit, backup_pos),
        b'\t' | b' '..=b'~' => append(result, &[char::from(to_append)], on_line, limit, backup_pos),
        _ => append(
            result,
            &hex_encode_byte(to_append),
            on_line,
            limit,
            backup_pos,
        ),
    }
}

#[inline(always)]
fn hex_encode_byte(byte: u8) -> [char; 3] {
    [
        '=',
        lower_nibble_to_hex(byte >> 4),
        lower_nibble_to_hex(byte),
    ]
}

#[inline(always)]
fn lower_nibble_to_hex(half_byte: u8) -> char {
    HEX_CHARS[(half_byte & 0x0F) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        assert_eq!(
            "hello world",
            lib::String::from_utf8(decode("hello world", ParseMode::Strict).unwrap()).unwrap()
        );
        assert_eq!(
            "Now's the time for all folk to come to the aid of their country.",
            lib::String::from_utf8(
                decode(
                    "Now's the time =\r\nfor all folk to come=\r\n \
                                                 to the aid of their country.",
                    ParseMode::Strict,
                )
                .unwrap(),
            )
            .unwrap()
        );
        assert_eq!(
            "\r\nhello=world",
            lib::String::from_utf8(decode("=0D=0Ahello=3Dworld", ParseMode::Strict).unwrap())
                .unwrap()
        );
        assert_eq!(
            "hello world\r\ngoodbye world",
            lib::String::from_utf8(
                decode("hello world\r\ngoodbye world", ParseMode::Strict).unwrap(),
            )
            .unwrap()
        );
        assert_eq!(
            "hello world\r\ngoodbye world",
            lib::String::from_utf8(
                decode("hello world   \r\ngoodbye world   ", ParseMode::Strict).unwrap(),
            )
            .unwrap()
        );
        assert_eq!(
            "hello world\r\ngoodbye world x",
            lib::String::from_utf8(
                decode(
                    "hello world   \r\ngoodbye world =  \r\nx",
                    ParseMode::Strict,
                )
                .unwrap(),
            )
            .unwrap()
        );

        assert_eq!(true, decode("hello world=x", ParseMode::Strict).is_err());
        assert_eq!(
            "hello world=x",
            lib::String::from_utf8(decode("hello world=x", ParseMode::Robust).unwrap()).unwrap()
        );

        assert_eq!(true, decode("hello =world=", ParseMode::Strict).is_err());
        assert_eq!(
            "hello =world",
            lib::String::from_utf8(decode("hello =world=", ParseMode::Robust).unwrap()).unwrap()
        );

        assert_eq!(true, decode("hello world=3d", ParseMode::Strict).is_err());
        assert_eq!(
            "hello world=",
            lib::String::from_utf8(decode("hello world=3d", ParseMode::Robust).unwrap()).unwrap()
        );

        assert_eq!(true, decode("hello world=3m", ParseMode::Strict).is_err());
        assert_eq!(
            "hello world=3m",
            lib::String::from_utf8(decode("hello world=3m", ParseMode::Robust).unwrap()).unwrap()
        );

        assert_eq!(true, decode("hello\u{FF}world", ParseMode::Strict).is_err());
        assert_eq!(
            "helloworld",
            lib::String::from_utf8(decode("hello\u{FF}world", ParseMode::Robust).unwrap()).unwrap()
        );

        assert_eq!(
            true,
            decode(
                "12345678901234567890123456789012345678901234567890123456789012345678901234567",
                ParseMode::Strict,
            )
            .is_err()
        );
        assert_eq!(
            "12345678901234567890123456789012345678901234567890123456789012345678901234567",
            lib::String::from_utf8(
                decode(
                    "12345678901234567890123456789012345678901234567890123456789012345678901234567",
                    ParseMode::Robust,
                )
                .unwrap(),
            )
            .unwrap()
        );
        assert_eq!(
            "1234567890123456789012345678901234567890123456789012345678901234567890123456",
            lib::String::from_utf8(
                decode(
                    "1234567890123456789012345678901234567890123456789012345678901234567890123456",
                    ParseMode::Strict,
                )
                .unwrap(),
            )
            .unwrap()
        );
    }

    #[test]
    fn test_encode() {
        assert_eq!("hello, world!", encode_to_str("hello, world!".as_bytes()));
        assert_eq!(
            "hello,=0Cworld!",
            encode_to_str("hello,\u{c}world!".as_bytes())
        );
        assert_eq!(
            "this=00is=C3=BFa=3Dlong=0Dstring=0Athat gets wrapped and stuff, \
                    woohoo!=C3=\r\n=89",
            encode_to_str(
                "this\u{0}is\u{FF}a=long\rstring\nthat gets \
                                             wrapped and stuff, woohoo!\u{c9}",
            )
        );
        assert_eq!(
            "this=00is=C3=BFa=3Dlong=0Dstring=0Athat just fits in a line,   woohoo!=C3=89",
            encode_to_str(
                "this\u{0}is\u{FF}a=long\rstring\nthat just fits \
                                             in a line,   woohoo!\u{c9}",
            )
        );
        assert_eq!(
            "this=20\r\nhas linebreaks\r\n built right in.",
            encode_to_str("this \r\nhas linebreaks\r\n built right in.")
        );
        // Test that soft line breaks get inserted at the right place
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXY",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXY",
            )
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\nXY",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXY",
            )
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\nXXY",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXY",
            )
        );
        // Test that soft line breaks don't break up an encoded octet
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=00Y",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\u{0}Y",
            )
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\n=00Y",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\u{0}Y",
            )
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\n=00Y",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\u{0}Y",
            )
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\n=00Y",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\u{0}Y",
            )
        );
        assert_eq!("=0D=3D", encode_to_str("\r="));
        assert_eq!("=0D\r\n", encode_to_str("\r\r\n"));
        assert_eq!("a=0D\r\nb", encode_to_str("a\r\r\nb"));
        assert_eq!("=0D", encode_to_str("\r"));
        assert_eq!("=0D=0D", encode_to_str("\r\r"));
        assert_eq!("\r\n", encode_to_str("\r\n"));

        assert_eq!("trailing spaces =20", encode_to_str("trailing spaces  "),);
        assert_eq!(
            "trailing spaces and crlf =20\r\n",
            encode_to_str("trailing spaces and crlf  \r\n"),
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\n=09",
            encode_to_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX\t"
            ),
        );
        assert_eq!(
            "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=\r\n=20\r\n",
            encode_to_str("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX \r\n"),
        );
    }

    #[test]
    fn test_lower_nibble_to_hex() {
        let test_data: &[(u8, char, char)] = &[
            (0, '0', '0'),
            (1, '0', '1'),
            (9, '0', '9'),
            (10, '0', 'A'),
            (15, '0', 'F'),
            (16, '1', '0'),
            (255, 'F', 'F'),
        ];

        for &(nr, high, low) in test_data.iter() {
            let got_high = lower_nibble_to_hex(nr >> 4);
            assert_eq!(high, got_high);
            let got_low = lower_nibble_to_hex(nr);
            assert_eq!(low, got_low);
        }
    }

    // from https://github.com/staktrace/quoted-printable/issues/13
    #[test]
    fn test_qp_rt() {
        let s = b"foo\r\n";
        let qp = encode_to_str(s);
        let rt = decode(&qp, ParseMode::Strict).unwrap();
        assert_eq!(s.as_slice(), rt.as_slice());
    }

    #[test]
    fn test_binary() {
        assert_eq!("foo=0D=0A", encode_binary_to_str("foo\r\n"));
        assert_eq!(
            "foo\r\n",
            lib::String::from_utf8(decode("foo=0D=0A", ParseMode::Strict).unwrap()).unwrap()
        );

        assert_eq!(
            "=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=0A=0D=\r\n=0A=0D=0A=0D=0A",
            encode_binary_to_str("\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n")
        );
    }

    #[test]
    fn test_three() {
        // this test enters the fast path for encoding runs of
        // characters that don't need encoding with three characters
        // left on the line and the next character needing encoding -
        // checks for potential off-by-one mistake in that loop
        assert_eq!(
	    "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=3DX=\r\n=3D=3DY",
            encode_to_str(
		"XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=X==Y",
            )
        );
    }
}
