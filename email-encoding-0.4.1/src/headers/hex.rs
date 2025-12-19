const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

pub(super) const fn encode_byte(byte: u8) -> [u8; 2] {
    [lower_nibble_to_hex(byte >> 4), lower_nibble_to_hex(byte)]
}

const fn lower_nibble_to_hex(half_byte: u8) -> u8 {
    HEX_CHARS[(half_byte & 0x0F) as usize]
}
