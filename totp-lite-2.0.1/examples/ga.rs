//! A Google Authenticator compatible TOTP implementation.
//! Technical reference:
//! https://en.wikipedia.org/wiki/Google_Authenticator#Technical_description
//!
//! In order to keep this example short it has no error handling, and
//! therefore it can panic, for example if you enter an invalid base32
//! character.
//!
//! You can run this example as follows:
//! ```sh
//!   cargo run --example ga
//! ```

use koibumi_base32 as base32;
use std::io::{self, Write};
use std::time::SystemTime;
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};

fn main() {
    println!("Press ctrl-c to cancel.");
    loop {
        let mut input = String::new();

        // Request the TOTP secret.
        print!("Enter your TOTP secret: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        let length = input.trim().len();
        if length != 16 && length != 26 && length != 32 {
            println!("Invalid TOTP secret, must be 16, 26 or 32 characters.");
            continue;
        }

        // The number of seconds since the Unix Epoch, used to calcuate a TOTP secret.
        let seconds: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate a 6 digit TOTP two-factor authentication code.
        println!(
            "Your TOTP code: {}",
            totp_custom::<Sha1>(
                // Calculate a new code every 30 seconds.
                DEFAULT_STEP,
                // Calculate a 6 digit code.
                6,
                // Convert the secret into bytes using base32::decode().
                &base32::decode(&input.trim().to_lowercase()).unwrap(),
                // Seconds since the Unix Epoch.
                seconds,
            )
        );
    }
}
