use std::io::stdin;
use yubico_ng::config::Config;
use yubico_ng::verify;

fn main() {
    println!("Please plug in a yubikey and enter an OTP");

    let client_id = std::env::var("YK_CLIENT_ID")
        .expect("Please set a value to the YK_CLIENT_ID environment variable.");

    let api_key = std::env::var("YK_API_KEY")
        .expect("Please set a value to the YK_API_KEY environment variable.");

    let api_host = std::env::var("YK_API_HOST")
        .expect("Please set a value to the YK_API_HOST environment variable.");

    let config = Config::default()
        .set_client_id(client_id)
        .set_key(api_key)
        .set_api_hosts(vec![api_host]);

    let otp = read_user_input();

    match verify(otp, config) {
        Ok(answer) => println!("{}", answer),
        Err(e) => println!("Error: {}", e),
    }
}

fn read_user_input() -> String {
    let mut buf = String::new();
    stdin()
        .read_line(&mut buf)
        .expect("Could not read user input.");

    buf
}
