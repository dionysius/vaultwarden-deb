use crate::yubicoerror::YubicoError;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{digest::CtOutput, Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

//  1. Apply the HMAC-SHA-1 algorithm on the line as an octet string using the API key as key
pub fn build_signature(key: &[u8], input: &[u8]) -> Result<CtOutput<HmacSha1>, YubicoError> {
    let decoded_key = STANDARD.decode(key)?;

    let mut hmac = match HmacSha1::new_from_slice(&decoded_key) {
        Ok(h) => h,
        Err(_) => return Err(YubicoError::InvalidKeyLength),
    };
    hmac.update(input);
    Ok(hmac.finalize())
}

pub fn verify_signature(key: &[u8], input: &[u8], expected: &[u8]) -> Result<(), YubicoError> {
    let decoded_key = STANDARD.decode(key)?;

    let mut hmac = match HmacSha1::new_from_slice(&decoded_key) {
        Ok(h) => h,
        Err(_) => return Err(YubicoError::InvalidKeyLength),
    };
    hmac.update(input);
    hmac.verify_slice(expected)
        .map_err(|_| YubicoError::SignatureMismatch)
}
