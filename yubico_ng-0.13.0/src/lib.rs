#[cfg(feature = "online-tokio")]
pub mod async_verifier;
pub mod config;
mod sec;
pub mod sync_verifier;
pub mod yubicoerror;

use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use config::Config;
use rand::distr::{Alphanumeric, SampleString};
use yubicoerror::YubicoError;

#[cfg(feature = "online-tokio")]
pub use async_verifier::verify_async;
pub use sync_verifier::verify;

type Result<T> = ::std::result::Result<T, YubicoError>;

#[derive(Clone)]
pub struct Request {
    query: String,
    response_verifier: ResponseVerifier,
}

impl Request {
    fn build_url(&self, for_api_host: &str) -> String {
        format!("{}?{}", for_api_host, self.query)
    }
}

#[derive(Clone)]
pub struct ResponseVerifier {
    otp: String,
    nonce: String,
    key: Vec<u8>,
}

impl ResponseVerifier {
    fn verify_response(&self, raw_response: String) -> Result<()> {
        let response_map: BTreeMap<String, String> = build_response_map(raw_response);

        if let Some(status) = response_map.get("status") {
            if status == "OK" {
                // Signature located in the response must match the signature we will build
                let signature_response: &str = response_map
                    .get("h")
                    .ok_or_else(|| YubicoError::InvalidResponse)?;
                verify_signature(signature_response, response_map.clone(), &self.key)?;

                // Check if "otp" in the response is the same as the "otp" supplied in the request.
                let otp_response: &str = response_map
                    .get("otp")
                    .ok_or_else(|| YubicoError::InvalidResponse)?;
                if !self.otp.eq(otp_response) {
                    return Err(YubicoError::OTPMismatch);
                }

                // Check if "nonce" in the response is the same as the "nonce" supplied in the request.
                let nonce_response: &str = response_map
                    .get("nonce")
                    .ok_or_else(|| YubicoError::InvalidResponse)?;
                if !self.nonce.eq(nonce_response) {
                    return Err(YubicoError::NonceMismatch);
                }

                Ok(())
            } else {
                // Check the status of the operation
                match status.as_str() {
                    "BAD_OTP" => Err(YubicoError::BadOTP),
                    "REPLAYED_OTP" => Err(YubicoError::ReplayedOTP),
                    "BAD_SIGNATURE" => Err(YubicoError::BadSignature),
                    "MISSING_PARAMETER" => Err(YubicoError::MissingParameter),
                    "NO_SUCH_CLIENT" => Err(YubicoError::NoSuchClient),
                    "OPERATION_NOT_ALLOWED" => Err(YubicoError::OperationNotAllowed),
                    "BACKEND_ERROR" => Err(YubicoError::BackendError),
                    "NOT_ENOUGH_ANSWERS" => Err(YubicoError::NotEnoughAnswers),
                    "REPLAYED_REQUEST" => Err(YubicoError::ReplayedRequest),
                    _ => Err(YubicoError::UnknownStatus),
                }
            }
        } else {
            Err(YubicoError::UnknownStatus)
        }
    }
}

fn build_request<S>(otp: S, config: &Config) -> Result<Request>
where
    S: Into<String>,
{
    let str_otp = otp.into();

    // A Yubikey can be configured to add line ending chars, or not.
    let str_otp = str_otp.trim().to_string();

    if printable_characters(&str_otp) {
        let nonce: String = generate_nonce();
        let mut query = form_urlencoded::Serializer::new(String::new());
        query.append_pair("id", &config.client_id);
        query.append_pair("nonce", &nonce);
        query.append_pair("otp", &str_otp);
        query.append_pair("sl", &config.sync_level.to_string());

        let query = query.finish();
        match sec::build_signature(&config.key, query.as_bytes()) {
            Ok(signature) => {
                // Recover the query
                let mut query = form_urlencoded::Serializer::new(query);

                // Base 64 encode the resulting value according to RFC 4648
                let encoded_signature = STANDARD.encode(signature.into_bytes());

                // Append the value under key h to the message.
                query.append_pair("h", &encoded_signature);

                let verifier = ResponseVerifier {
                    otp: str_otp,
                    nonce,
                    key: config.key.clone(),
                };

                let request = Request {
                    query: query.finish(),
                    response_verifier: verifier,
                };

                Ok(request)
            }
            Err(error) => Err(error),
        }
    } else {
        Err(YubicoError::BadOTP)
    }
}

// Recommendation is that clients only check that the input consists of 32-48 printable characters
fn printable_characters(otp: &str) -> bool {
    for c in otp.chars() {
        if !c.is_ascii() {
            return false;
        }
    }
    otp.len() > 32 && otp.len() < 48
}

fn generate_nonce() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 40)
}

// Remove the signature itself from the values over for verification.
// Sort the key/value pairs.
fn verify_signature(
    signature_response: &str,
    mut response_map: BTreeMap<String, String>,
    key: &[u8],
) -> Result<()> {
    response_map.remove("h");

    let mut query = String::new();
    for (key, value) in response_map {
        let param = format!("{}={}&", key, value);
        query.push_str(param.as_ref());
    }
    query.pop(); // remove last &

    let decoded_signature = &STANDARD.decode(signature_response).unwrap()[..];
    sec::verify_signature(key, query.as_bytes(), decoded_signature)
}

fn build_response_map(result: String) -> BTreeMap<String, String> {
    let mut parameters = BTreeMap::new();
    for line in result.lines() {
        let param: Vec<&str> = line.splitn(2, '=').collect();
        if param.len() > 1 {
            parameters.insert(param[0].to_string(), param[1].to_string());
        }
    }
    parameters
}
