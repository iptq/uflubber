use irc_async::Config as IrcConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    host: String,
    nick: String,
    port: u16,
    ssl: bool,
}

impl From<&Config> for IrcConfig {
    fn from(config: &Config) -> Self {
        IrcConfig {
            host: config.host.clone(),
            nick: config.nick.clone(),
            port: config.port,
            ssl: config.ssl,
        }
    }
}
