#[macro_use]
extern crate anyhow;

use std::path::PathBuf;

use anyhow::Result;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use irc_async::{Client, Config as IrcConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use structopt::StructOpt;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{BytesCodec, FramedWrite};
use toml::Value as TomlValue;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,

    #[structopt(long = "plugin-name")]
    plugin_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let config: TomlValue = {
        let mut config_file = File::open(&args.config_path).await?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).await?;
        toml::from_str(&contents)?
    };

    let plugin_config = config
        .get("plugins")
        .ok_or_else(|| anyhow!("No 'plugins' under config.toml"))?
        .get(&args.plugin_name)
        .ok_or_else(|| anyhow!("Plugin name '{}' not found", args.plugin_name))?
        .get("config")
        .ok_or_else(|| anyhow!("Plugin has no config"))?
        .clone()
        .try_into::<Config>()?;
    let irc_config = IrcConfig::from(&plugin_config);

    let (mut client, fut) = Client::with_config(irc_config).await?;
    client.register().await?;

    tokio::spawn(fut);

    // main loop
    let mut stdout = Framed::<_, JsonValue, JsonValue, _>::new(
        FramedWrite::new(io::stdout(), BytesCodec::new()),
        Json::<JsonValue, JsonValue>::default(),
    );
    async {
        while let Some(Ok(message)) = client.next().await {
            // println!("message: {:?}", message);
            // println!("{{}}");
            stdout.send(json!({ "a": "b" })).await;
        }
    }
        .await;

    Ok(())
}