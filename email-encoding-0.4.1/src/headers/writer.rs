//! Utilities for writing email headers to a [`Write`]r.
//!
//! [`Write`]: core::fmt::Write

use core::fmt::{self, Write};

use super::MAX_LINE_LEN;

/// Wrapper around [`Write`] that remembers the length of the
/// last line written to it.
///
/// [`Write`]: core::fmt::Write
pub struct EmailWriter<'a> {
    writer: &'a mut dyn Write,
    line_len: usize,
    spaces: usize,
    can_go_to_new_line_now: bool,
}

impl<'a> EmailWriter<'a> {
    /// Construct a new `EmailWriter`.
    ///
    /// * `line_len` is the length of the last line in `writer`.
    /// * `spaces` the number of spaces that must be written before
    ///   the next write.
    /// * `can_go_to_new_line_now` is whether the current line can
    ///   be wrapped now or not.
    pub fn new(
        writer: &'a mut dyn Write,
        line_len: usize,
        spaces: usize,
        can_go_to_new_line_now: bool,
    ) -> Self {
        Self {
            writer,
            line_len,
            spaces,
            can_go_to_new_line_now,
        }
    }

    /// Go to a new line and reset the `line_len` to `0`.
    pub fn new_line(&mut self) -> fmt::Result {
        self.writer.write_str("\r\n")?;
        self.line_len = 0;
        self.can_go_to_new_line_now = false;

        Ok(())
    }

    /// Write a space which _might_ get wrapped to a new line on the next write.
    pub fn space(&mut self) {
        self.spaces += 1;
    }

    /// Forget all buffered spaces
    pub(super) fn forget_spaces(&mut self) {
        self.spaces = 0;
    }

    pub(super) fn has_spaces(&mut self) -> bool {
        self.spaces >= 1
    }

    /// Get the length in bytes of the last line written to the inner writer.
    pub fn line_len(&self) -> usize {
        self.line_len
    }

    /// Get the length in bytes of the last line written to the inner writer
    /// plus the spaces which might be written to in on the next write call.
    pub fn projected_line_len(&self) -> usize {
        self.line_len + self.spaces
    }

    /// Get a [`Write`]r which automatically line folds text written to it.
    ///
    /// [`Write`]: core::fmt::Write
    pub fn folding<'b>(&'b mut self) -> FoldingEmailWriter<'a, 'b> {
        FoldingEmailWriter { writer: self }
    }

    fn write_spaces(&mut self) -> fmt::Result {
        while self.spaces > 0 {
            self.writer.write_char(' ')?;
            self.line_len += 1;
            self.spaces -= 1;
        }

        Ok(())
    }
}

impl Write for EmailWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_spaces()?;

        let s_after = s.trim_end_matches(' ');
        self.spaces += s.len() - s_after.len();

        if !s_after.is_empty() {
            self.writer.write_str(s_after)?;
            self.line_len += s_after.len();
            self.can_go_to_new_line_now = true;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if c == ' ' {
            self.spaces += 1;
        } else {
            self.write_spaces()?;
            self.can_go_to_new_line_now = true;

            self.writer.write_char(c)?;
            self.line_len += c.len_utf8();
        }

        Ok(())
    }
}

impl Drop for EmailWriter<'_> {
    fn drop(&mut self) {
        let _ = self.write_spaces();
    }
}

/// Wrapper around [`Write`] that remembers the length of the
/// last line and automatically line folds text written to it.
///
/// [`Write`]: core::fmt::Write
pub struct FoldingEmailWriter<'a, 'b> {
    writer: &'b mut EmailWriter<'a>,
}

impl Write for FoldingEmailWriter<'_, '_> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            if s.starts_with(' ') {
                self.writer.space();
                s = &s[1..];
                continue;
            }

            let (start, end) = s.find(' ').map_or((s, ""), |i| s.split_at(i));

            if self.writer.can_go_to_new_line_now
                && self.writer.spaces >= 1
                && (self.writer.projected_line_len() + start.len()) > MAX_LINE_LEN
            {
                self.writer.new_line()?;
            }

            self.writer.write_str(start)?;
            s = end;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if c == ' ' {
            self.writer.spaces += 1;
        } else {
            self.write_str(c.encode_utf8(&mut [0u8; 4]))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn wrap_immediate() {
        let mut s =
            "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            for _ in 0..16 {
                w.folding().write_str("0123456789").unwrap();
            }
        }

        assert_eq!(
            s,
            "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
        );
    }

    #[test]
    fn wrap_keeping_final_whitespace() {
        let mut s = "Subject: AAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("12345 ").unwrap();
            w.new_line().unwrap();
            w.folding().write_str("12345").unwrap();
        }

        assert_eq!(s, concat!("Subject: AAAAAAAAAAAAAA 12345\r\n", " 12345"));
    }

    #[test]
    fn catch_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB ").unwrap();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                " CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn catch_spaces() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB   ").unwrap();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                "   CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn explicit_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB").unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                " CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn explicit_spaces() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 1, true);
            w.folding().write_str("BBB").unwrap();
            w.space();
            w.write_char(' ').unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBB\r\n",
                "   CCCCCCCCCCCCC"
            )
        );
    }

    #[test]
    fn optional_breakpoint() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
            .to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.space();
            w.folding().write_str("BBBBBBBBBB").unwrap();
            w.space();
            w.folding().write_str("CCCCCCCCCC").unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\r\n",
                " BBBBBBBBBB CCCCCCCCCC",
            )
        );
    }

    #[test]
    fn double_spaces_issue_949() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA ".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.folding().write_str("BBBBBBBBBBBBB ").unwrap();
            crate::headers::rfc2047::encode("sélection", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBBBBBBBBBBBB\r\n",
                " =?utf-8?b?c8OpbGVjdGlvbg==?=",
            )
        );
    }

    #[test]
    fn double_spaces_issue_949_no_space() {
        let mut s = "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA ".to_owned();
        let line_len = s.len();

        {
            let mut w = EmailWriter::new(&mut s, line_len, 0, true);
            w.folding().write_str("BBBBBBBBBBBBBBB").unwrap();
            crate::headers::rfc2047::encode("sélection", &mut w).unwrap();
        }

        assert_eq!(
            s,
            concat!(
                "Subject: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA BBBBBBBBBBBBBBB=?utf-8?b?cw==?=\r\n",
                " =?utf-8?b?w6lsZWN0aW9u?=",
            )
        );
    }
}
