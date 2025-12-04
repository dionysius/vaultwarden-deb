use std::fmt::Display;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Slot1,
    Slot2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Sha1,
    Otp,
}

/// From the Validation Protocol documentation:
///
/// A value 0 to 100 indicating percentage of syncing required by client,
/// or strings "fast" or "secure" to use server-configured values; if
/// absent, let the server decide.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SyncLevel(u8);

impl SyncLevel {
    pub fn fast() -> SyncLevel {
        SyncLevel(0)
    }

    pub fn secure() -> SyncLevel {
        SyncLevel(100)
    }

    pub fn custom(level: u8) -> SyncLevel {
        if level > 100 {
            SyncLevel(100)
        } else {
            SyncLevel(level)
        }
    }
}

impl Display for SyncLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub client_id: String,
    pub key: Vec<u8>,
    pub api_hosts: Vec<String>,
    pub user_agent: String,
    pub sync_level: SyncLevel,
    /// The timeout for HTTP requests.
    pub request_timeout: Duration,
    pub proxy_url: String,
    pub proxy_username: String,
    pub proxy_password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            key: Vec::new(),
            api_hosts: vec!["https://api.yubico.com/wsapi/2.0/verify".to_string()],
            user_agent: "github.com/BlackDex/yubico-rs".to_string(),
            sync_level: SyncLevel::secure(),
            request_timeout: Duration::from_secs(30), // Value taken from the reqwest crate.
            proxy_url: String::new(),
            proxy_username: String::new(),
            proxy_password: String::new(),
        }
    }
}

impl Config {
    pub fn set_client_id<C>(mut self, client_id: C) -> Self
    where
        C: Into<String>,
    {
        self.client_id = client_id.into();
        self
    }

    pub fn set_key<K>(mut self, key: K) -> Self
    where
        K: Into<String>,
    {
        self.key = key.into().into_bytes();
        self
    }

    pub fn set_api_hosts(mut self, hosts: Vec<String>) -> Self {
        self.api_hosts = hosts;
        self
    }

    pub fn set_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn set_sync_level(mut self, level: SyncLevel) -> Self {
        self.sync_level = level;
        self
    }

    pub fn set_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    pub fn set_proxy_url<P>(mut self, proxy_url: P) -> Self
    where
        P: Into<String>,
    {
        self.proxy_url = proxy_url.into();
        self
    }

    pub fn set_proxy_username<U>(mut self, proxy_username: U) -> Self
    where
        U: Into<String>,
    {
        self.proxy_username = proxy_username.into();
        self
    }

    pub fn set_proxy_password<P>(mut self, proxy_password: P) -> Self
    where
        P: Into<String>,
    {
        self.proxy_password = proxy_password.into();
        self
    }
}
