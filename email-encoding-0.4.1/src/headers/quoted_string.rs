//! Quoted String encoder.

use core::fmt::{self, Write};

use super::{rfc2047, utils, writer::EmailWriter};

/// Encode a string that may need to be quoted.
///
/// # Examples
///
/// ```rust
/// # use email_encoding::headers::writer::EmailWriter;
/// # fn main() -> core::fmt::Result {
/// {
///     let input = "John";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::quoted_string::encode(input, &mut writer)?;
///     }
///     assert_eq!(output, "John");
/// }
///
/// {
///     let input = "John Smith";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::quoted_string::encode(input, &mut writer)?;
///     }
///     assert_eq!(output, "\"John Smith\"");
/// }
///
/// {
///     let input = "Rogue \" User";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::quoted_string::encode(input, &mut writer)?;
///     }
///     assert_eq!(output, "\"Rogue \\\" User\"");
/// }
///
/// {
///     let input = "Adrián";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::quoted_string::encode(input, &mut writer)?;
///     }
///     assert_eq!(output, "=?utf-8?b?QWRyacOhbg==?=");
/// }
/// # Ok(())
/// # }
/// ```
pub fn encode(value: &str, w: &mut EmailWriter<'_>) -> fmt::Result {
    #[derive(Debug)]
    enum Strategy {
        Plain,
        Quoted,
        QuotedEscaped,
        Rfc2047,
    }

    let mut strategy = Strategy::Plain;

    let mut bytes = value.as_bytes();

    // Plain -> Quoted
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric() && !matches!(byte, b'-' | b'_' | b'.') {
            strategy = Strategy::Quoted;
            break;
        }

        bytes = &bytes[1..];
    }

    // Quoted -> QuotedEscaped
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric() && !matches!(byte, b' ' | b'-' | b'_' | b'.') {
            strategy = Strategy::QuotedEscaped;
            break;
        }

        bytes = &bytes[1..];
    }

    // QuotedEscaped -> Rfc2047
    while !bytes.is_empty() {
        let byte = bytes[0];

        if !byte.is_ascii_alphanumeric()
            && !matches!(byte, b'\\' | b'"' | b' ' | b'-' | b'_' | b'.')
        {
            strategy = Strategy::Rfc2047;
            break;
        }

        bytes = &bytes[1..];
    }

    match strategy {
        Strategy::Plain => {
            w.write_str(value)?;
        }
        Strategy::Quoted => {
            w.write_char('"')?;
            w.folding().write_str(value)?;
            w.write_char('"')?;
        }
        Strategy::QuotedEscaped => {
            w.write_char('"')?;
            utils::write_escaped(value, &mut w.folding())?;
            w.write_char('"')?;
        }
        Strategy::Rfc2047 => {
            rfc2047::encode(value, w)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn plain() {
        let mut s = String::new();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            encode("1234567890abcd", &mut w).unwrap();
        }

        assert_eq!(s, "1234567890abcd");
    }

    #[test]
    fn quoted() {
        let mut s = String::new();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            encode("1234567890 abcd", &mut w).unwrap();
        }

        assert_eq!(s, "\"1234567890 abcd\"");
    }

    #[test]
    fn quoted_long() {
        let mut s = String::new();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            encode("1234567890 abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd", &mut w).unwrap();
        }

        assert_eq!(s, concat!(
            "\"1234567890\r\n",
            " abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd\""
        ));
    }

    #[test]
    fn quoted_escaped() {
        let mut s = String::new();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            encode("12345\\67890 ab\"cd", &mut w).unwrap();
        }

        assert_eq!(s, "\"12345\\\\67890 ab\\\"cd\"");
    }

    // TODO: get it working for the quoted escaped strategy
    // #[test]
    // fn quoted_escaped_long() {
    //     let mut s = String::new();
    //     let line_len = s.len();
    //
    //     {
    //         let mut w = EmailWriter::new(&mut s, line_len, 0, false, false);
    //         encode("12345\\67890 ab\"cdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd", &mut w).unwrap();
    //     }
    //
    //     assert_eq!(s, concat!(
    //         "\"12345\\\\67890\r\n",
    //         " ab\\\"cdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd\""
    //     ));
    // }

    #[test]
    fn rfc2047() {
        let mut s = String::new();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, false);
            encode("12345\\67890 perché ab\"cd", &mut w).unwrap();
        }

        assert_eq!(s, "=?utf-8?b?MTIzNDVcNjc4OTAgcGVyY2jDqSBhYiJjZA==?=");
    }
}
