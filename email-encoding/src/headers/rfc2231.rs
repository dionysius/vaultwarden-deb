//! [RFC 2231] encoder.
//!
//! [RFC 2231]: https://datatracker.ietf.org/doc/html/rfc2231

use core::fmt::{self, Write};

use super::{hex_encoding, utils, writer::EmailWriter, MAX_LINE_LEN};

/// Encode a string via RFC 2231.
///
/// # Examples
///
/// ```rust
/// # use email_encoding::headers::writer::EmailWriter;
/// # fn main() -> core::fmt::Result {
/// {
///     let input = "invoice.pdf";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::rfc2231::encode("filename", input, &mut writer)?;
///     }
///     assert_eq!(output, "filename=\"invoice.pdf\"");
/// }
///
/// {
///     let input = "invoice_2022_06_04_letshaveaverylongfilenamewhynotemailcanhandleit.pdf";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::rfc2231::encode("filename", input, &mut writer)?;
///     }
///     assert_eq!(
///         output,
///         concat!(
///             "\r\n",
///             " filename*0=\"invoice_2022_06_04_letshaveaverylongfilenamewhynotemailcanha\";\r\n",
///             " filename*1=\"ndleit.pdf\""
///         )
///     );
/// }
///
/// {
///     let input = "faktúra.pdf";
///
///     let mut output = String::new();
///     {
///         let mut writer = EmailWriter::new(&mut output, 0, 0, false);
///         email_encoding::headers::rfc2231::encode("filename", input, &mut writer)?;
///     }
///     assert_eq!(
///         output,
///         concat!(
///             "\r\n",
///             " filename*0*=utf-8''fakt%C3%BAra.pdf"
///         )
///     );
/// }
/// # Ok(())
/// # }
/// ```
pub fn encode(key: &str, mut value: &str, w: &mut EmailWriter<'_>) -> fmt::Result {
    assert!(
        utils::str_is_ascii_alphanumeric(key),
        "`key` must only be composed of ascii alphanumeric chars"
    );
    assert!(
        key.len() + "*12*=utf-8'';".len() < MAX_LINE_LEN,
        "`key` must not be too long to cause the encoder to overflow the max line length"
    );

    if utils::str_is_ascii_printable(value) {
        // Can be written normally (Parameter Value Continuations)

        let quoted_plain_combined_len = key.len() + "=\"".len() + value.len() + "\"\r\n".len();
        if w.line_len() + quoted_plain_combined_len <= MAX_LINE_LEN {
            // Fits line

            w.write_str(key)?;

            w.write_char('=')?;

            w.write_char('"')?;
            utils::write_escaped(value, w)?;
            w.write_char('"')?;
        } else {
            // Doesn't fit line

            w.new_line()?;
            w.forget_spaces();

            let mut i = 0_usize;
            loop {
                write!(w, " {}*{}=\"", key, i)?;

                let remaining_len = MAX_LINE_LEN - w.line_len() - "\"\r\n".len();

                let value_ =
                    utils::truncate_to_char_boundary(value, remaining_len.min(value.len()));
                value = &value[value_.len()..];

                utils::write_escaped(value_, w)?;

                w.write_char('"')?;

                if value.is_empty() {
                    // End of value
                    break;
                }

                // End of line
                w.write_char(';')?;
                w.new_line()?;

                i += 1;
            }
        }
    } else {
        // Needs encoding (Parameter Value Character Set and Language Information)

        w.new_line()?;
        w.forget_spaces();

        let mut i = 0_usize;
        loop {
            write!(w, " {}*{}*=", key, i)?;

            if i == 0 {
                w.write_str("utf-8''")?;
            }

            let mut chars = value.chars();
            while w.line_len() < MAX_LINE_LEN - "=xx=xx=xx=xx;\r\n".len() {
                match chars.next() {
                    Some(c) => {
                        hex_encoding::percent_encode_char(w, c)?;
                        value = chars.as_str();
                    }
                    None => {
                        break;
                    }
                }
            }

            if value.is_empty() {
                // End of value
                break;
            }

            // End of line
            w.write_char(';')?;
            w.new_line()?;

            i += 1;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::ToOwned, string::String};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = 1;

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode("filename", "", &mut w).unwrap();
        }

        assert_eq!(s, concat!("Content-Disposition: attachment; filename=\"\""));
    }

    #[test]
    fn parameter() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = 1;

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode("filename", "duck.txt", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment; filename=\"duck.txt\"")
        );
    }

    #[test]
    fn parameter_to_escape() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = 1;

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode("filename", "du\"ck\\.txt", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!("Content-Disposition: attachment; filename=\"du\\\"ck\\\\.txt\"")
        );
    }

    #[test]
    fn parameter_long() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode(
                "filename",
                "a-fairly-long-filename-just-to-see-what-happens-when-we-encode-it-will-the-client-be-able-to-handle-it.txt",
                &mut w,
            )
            .unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0=\"a-fairly-long-filename-just-to-see-what-happens-when-we-enco\";\r\n",
                " filename*1=\"de-it-will-the-client-be-able-to-handle-it.txt\""
            )
        );
    }

    #[test]
    fn parameter_special() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode("filename", "caffè.txt", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''caff%C3%A8.txt"
            )
        );
    }

    #[test]
    fn parameter_special_long() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode(
                "filename",
                "testing-to-see-what-happens-when-📕📕📕📕📕📕📕📕📕📕📕-are-placed-on-the-boundary.txt",
                &mut w,
            )
            .unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''testing-to-see-what-happens-when-%F0%9F%93%95;\r\n",
                " filename*1*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*2*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*3*=%F0%9F%93%95%F0%9F%93%95-are-placed-on-the-bound;\r\n",
                " filename*4*=ary.txt"
            )
        );
    }

    #[test]
    fn parameter_special_long_part2() {
        let mut s = "Content-Disposition: attachment;".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            encode(
                "filename",
                "testing-to-see-what-happens-when-books-are-placed-in-the-second-part-📕📕📕📕📕📕📕📕📕📕📕.txt",
                &mut w,
            )
            .unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Content-Disposition: attachment;\r\n",
                " filename*0*=utf-8''testing-to-see-what-happens-when-books-ar;\r\n",
                " filename*1*=e-placed-in-the-second-part-%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*2*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*3*=%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95%F0%9F%93%95;\r\n",
                " filename*4*=%F0%9F%93%95.txt"
            )
        );
    }

    #[test]
    fn parameter_dont_split_on_hex_boundary() {
        let base_header = "Content-Disposition: attachment;".to_owned();
        let line_len = base_header.len();

        for start_offset in &["", "x", "xx", "xxx"] {
            let mut filename = (*start_offset).to_owned();

            for i in 1..256 {
                // 'Ü' results in two hex chars %C3%9C
                filename.push('Ü');

                let mut output = base_header.clone();
                {
                    let mut w = EmailWriter::new(&mut output, line_len, 0, true);
                    encode("filename", &filename, &mut w).unwrap();
                }

                // look for all hex encoded chars
                let output_len = output.len();
                let mut found_hex_count = 0;
                for (percent_sign_idx, _) in output.match_indices('%') {
                    assert!(percent_sign_idx + 3 <= output_len);

                    // verify we get the expected hex sequence for an 'Ü'
                    let must_be_hex = &output[percent_sign_idx + 1..percent_sign_idx + 3];
                    assert!(
                        must_be_hex == "C3" || must_be_hex == "9C",
                        "unexpected hex char: {}",
                        must_be_hex
                    );
                    found_hex_count += 1;
                }
                // verify the number of hex encoded chars adds up
                let number_of_chars_in_hex = 2;
                assert_eq!(found_hex_count, i * number_of_chars_in_hex);

                // verify max line length
                let mut last_newline_pos = 0;
                for (newline_idx, _) in output.match_indices("\r\n") {
                    let line_length = newline_idx - last_newline_pos;
                    assert!(
                        line_length < MAX_LINE_LEN,
                        "expected line length exceeded: {} > {}",
                        line_length,
                        MAX_LINE_LEN
                    );
                    last_newline_pos = newline_idx;
                }
                // ensure there was at least one newline
                assert_ne!(0, last_newline_pos);
            }
        }
    }

    #[test]
    #[should_panic(expected = "`key` must only be composed of ascii alphanumeric chars")]
    fn non_ascii_key() {
        let mut s = String::new();
        let mut w = EmailWriter::new(&mut s, 0, 0, true);
        let _ = encode("📬", "", &mut w);
    }
}
