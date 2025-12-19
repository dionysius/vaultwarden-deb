use core::fmt::{self, Write};

pub(super) fn str_is_ascii_alphanumeric(s: &str) -> bool {
    s.bytes().all(|c| c.is_ascii_alphanumeric())
}

pub(super) const fn char_is_ascii_alphanumeric_plus(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')
}

pub(super) fn str_is_ascii_printable(s: &str) -> bool {
    s.bytes().all(char_is_ascii_printable)
}

const fn char_is_ascii_printable(c: u8) -> bool {
    matches!(c, b' '..=b'~')
}

pub(super) fn write_escaped(s: &str, w: &mut impl Write) -> fmt::Result {
    debug_assert!(s.is_ascii());

    for b in s.bytes() {
        match b {
            b'\\' => {
                w.write_str("\\\\")?;
            }
            b'"' => {
                w.write_str("\\\"")?;
            }
            b => {
                w.write_char(char::from(b))?;
            }
        }
    }

    Ok(())
}

pub(super) fn truncate_to_char_boundary(s: &str, mut max: usize) -> &str {
    assert!(max <= s.len());

    while !s.is_char_boundary(max) {
        max -= 1;
    }
    &s[..max]
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate_to_char_boundary("12345678", 4), "1234");
    }

    #[test]
    fn truncate0_ascii() {
        assert_eq!(truncate_to_char_boundary("12345678", 0), "");
    }

    #[test]
    fn truncate_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 8), "📬📬");
    }

    #[test]
    fn truncate0_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 0), "");
    }

    #[test]
    fn truncate_boundary_utf8() {
        assert_eq!(truncate_to_char_boundary("📬📬📬📬📬📬", 9), "📬📬");
    }

    #[test]
    #[should_panic]
    fn truncate_out_of_bounds() {
        let _ = truncate_to_char_boundary("12345678", 16);
    }
}
