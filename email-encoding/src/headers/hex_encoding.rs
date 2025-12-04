use core::fmt::{self, Write};

use super::{hex, utils, writer::EmailWriter};

pub(super) fn percent_encode_char(w: &mut EmailWriter<'_>, to_append: char) -> fmt::Result {
    encode_char(w, '%', to_append)
}

fn encode_char(w: &mut EmailWriter<'_>, prefix: char, to_append: char) -> fmt::Result {
    if utils::char_is_ascii_alphanumeric_plus(to_append) {
        w.write_char(to_append)?;
    } else {
        let mut dst = [0; 4];
        let written = to_append.encode_utf8(&mut dst).len();

        encode_byte(w, prefix, dst[0])?;

        // Manually unrolled loop over `dst`
        if written >= 2 {
            encode_byte(w, prefix, dst[1])?;

            if written >= 3 {
                encode_byte(w, prefix, dst[2])?;

                if written >= 4 {
                    encode_byte(w, prefix, dst[3])?;
                }
            }
        }
    }

    Ok(())
}

fn encode_byte(w: &mut EmailWriter<'_>, prefix: char, to_append: u8) -> fmt::Result {
    let chars = hex::encode_byte(to_append);
    w.write_char(prefix)?;
    w.write_char(char::from(chars[0]))?;
    w.write_char(char::from(chars[1]))?;

    Ok(())
}
