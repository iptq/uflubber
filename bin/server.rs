use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;

use futures::future::{self, FutureExt};
use futures::stream::StreamExt;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::{self, fs::File, io::AsyncReadExt, process::Command};
use serde_json::Value;

use uflubber::JsonCodec;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    plugins: BTreeMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginConfig {
    path: PathBuf,
}

struct Server {

}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let config: Config = {
        let mut config_file = File::open(&args.config_path).await?;
        let mut contents = String::new();
        config_file.read_to_string(&mut contents).await?;
        toml::from_str(&contents)?
    };

    let server = Server {};

    let mut futures = Vec::new();
    for (name, plugin) in config.plugins.iter() {
        let mut cmd = Command::new(&plugin.path);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
        cmd.arg("--config")
            .arg(args.config_path.as_os_str())
            .arg("--plugin-name")
            .arg(&name);
        println!("spawning {}: {:?}", name, plugin);

        let mut child = cmd.spawn()?;
        let input = child.stdin().take().unwrap();
        let output = child.stdout().take().unwrap();

        futures.push(child);
    }

    tokio::spawn(future::join_all(futures));

    // Ok(future::join_all(futures).map(|_| ()).await)
    Ok(future::pending().await)
}
