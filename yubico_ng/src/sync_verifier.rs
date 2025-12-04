use std::io::Read;
use std::sync::mpsc::{channel, Sender};

use reqwest::header::USER_AGENT;
use threadpool::ThreadPool;

use crate::build_request;
use crate::config::Config;
use crate::yubicoerror::YubicoError;
use crate::Request;
use crate::Result;
use reqwest::blocking::Client;
use std::sync::Arc;

pub fn verify<S>(otp: S, config: Config) -> Result<String>
where
    S: Into<String>,
{
    Verifier::new(config)?.verify(otp)
}

pub struct Verifier {
    config: Config,
    thread_pool: ThreadPool,
    client: Arc<Client>,
}

impl Verifier {
    pub fn new(config: Config) -> Result<Verifier> {
        let number_of_hosts = config.api_hosts.len();
        let client = if !config.proxy_url.is_empty() && config.proxy_username.is_empty() {
            Verifier::get_client_proxy(config.clone())?
        } else if !config.proxy_url.is_empty() && !config.proxy_username.is_empty() {
            Verifier::get_client_proxy_with_auth(config.clone())?
        } else {
            Client::builder().timeout(config.request_timeout).build()?
        };

        Ok(Verifier {
            config,
            thread_pool: ThreadPool::new(number_of_hosts),
            client: Arc::new(client),
        })
    }

    #[allow(tail_expr_drop_order)]
    pub fn verify<S>(&self, otp: S) -> Result<String>
    where
        S: Into<String>,
    {
        let request = build_request(otp, &self.config)?;

        let number_of_hosts = self.config.api_hosts.len();
        let (tx, rx) = channel();

        for api_host in &self.config.api_hosts {
            let tx = tx.clone();
            let request = request.clone();
            let url = request.build_url(api_host);
            let user_agent = self.config.user_agent.clone();
            let client = Arc::clone(&self.client);

            self.thread_pool.execute(move || {
                process(&client, tx, url, &request, user_agent);
            });
        }

        let mut success = false;
        let mut results: Vec<Result<String>> = Vec::new();
        for _ in 0..number_of_hosts {
            match rx.recv() {
                Ok(Response::Signal(result)) => match result {
                    Ok(_) => {
                        results.truncate(0);
                        success = true;
                    }
                    Err(_) => {
                        results.push(result);
                    }
                },
                Err(e) => {
                    results.push(Err(YubicoError::ChannelError(e)));
                    break;
                }
            }
        }

        if success {
            Ok("The OTP is valid.".into())
        } else {
            results.pop().ok_or_else(|| YubicoError::InvalidOtp)?
        }
    }

    fn get_client_proxy(config: Config) -> Result<Client> {
        Ok(Client::builder()
            .timeout(config.request_timeout)
            .proxy(reqwest::Proxy::all(&config.proxy_url)?)
            .build()?)
    }

    fn get_client_proxy_with_auth(config: Config) -> Result<Client> {
        let proxy = reqwest::Proxy::all(&config.proxy_url)?
            .basic_auth(&config.proxy_username, &config.proxy_password);
        Ok(Client::builder()
            .timeout(config.request_timeout)
            .proxy(proxy)
            .build()?)
    }
}

enum Response {
    Signal(Result<String>),
}

fn process(
    client: &Client,
    sender: Sender<Response>,
    url: String,
    request: &Request,
    user_agent: String,
) {
    match get(client, url, user_agent) {
        Ok(raw_response) => {
            let result = request
                .response_verifier
                .verify_response(raw_response)
                .map(|()| "The OTP is valid.".to_owned());
            sender.send(Response::Signal(result)).unwrap();
        }
        Err(e) => {
            sender.send(Response::Signal(Err(e))).unwrap();
        }
    }
}

pub fn get(client: &Client, url: String, user_agent: String) -> Result<String> {
    let mut response = client
        .get(url.as_str())
        .header(USER_AGENT, user_agent)
        .send()?;

    let mut data = String::new();
    response.read_to_string(&mut data)?;

    Ok(data)
}
