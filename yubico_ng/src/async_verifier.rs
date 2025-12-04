use reqwest::header::USER_AGENT;
use reqwest::Client;

use crate::config::Config;
use crate::yubicoerror::YubicoError;
use crate::{build_request, Request, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;

pub async fn verify_async<S>(otp: S, config: Config) -> Result<()>
where
    S: Into<String>,
{
    AsyncVerifier::new(config)?.verify(otp).await
}

pub struct AsyncVerifier {
    client: Client,
    config: Config,
}

impl AsyncVerifier {
    pub fn new(config: Config) -> Result<AsyncVerifier> {
        let client = if !config.proxy_url.is_empty() && config.proxy_username.is_empty() {
            AsyncVerifier::get_client_proxy(config.clone())?
        } else if !config.proxy_url.is_empty() && !config.proxy_username.is_empty() {
            AsyncVerifier::get_client_proxy_with_auth(config.clone())?
        } else {
            Client::builder().timeout(config.request_timeout).build()?
        };

        Ok(AsyncVerifier { client, config })
    }

    #[allow(tail_expr_drop_order)]
    pub async fn verify<S>(&self, otp: S) -> Result<()>
    where
        S: Into<String>,
    {
        let request = Arc::new(build_request(otp, &self.config)?); // Arc because we need the future to be Send.

        let mut responses = FuturesUnordered::new();
        self.config
            .api_hosts
            .iter()
            .for_each(|api_host| responses.push(self.request(Arc::clone(&request), api_host)));

        let mut errors = vec![];

        while let Some(response) = responses.next().await {
            match response {
                Ok(()) => return Ok(()),
                Err(err @ YubicoError::ReplayedRequest) => errors.push(err),
                Err(YubicoError::HTTPStatusCode(code)) => {
                    errors.push(YubicoError::HTTPStatusCode(code))
                }
                Err(err) => return Err(err),
            }
        }

        Err(YubicoError::MultipleErrors(errors))
    }

    async fn request(&self, request: Arc<Request>, api_host: &str) -> Result<()> {
        let url = request.build_url(api_host);
        let http_request = self
            .client
            .get(&url)
            .header(USER_AGENT, self.config.user_agent.clone());

        let response = http_request.send().await?;
        let status_code = response.status();

        if !status_code.is_success() {
            return Err(YubicoError::HTTPStatusCode(status_code));
        }

        let text = response.text().await?;

        request.response_verifier.verify_response(text)
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
