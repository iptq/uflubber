use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Error;
use anyhow::Result;
use futures::{
    future::{self},
    stream::StreamExt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;
use tokio::{self, fs::File, io::AsyncReadExt, net::TcpListener, process::Command};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    bind_host: String,
    bind_port: u16,
    plugins: BTreeMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginConfig {
    path: PathBuf,
}

struct Backend {}

struct Server {
    backends: BTreeMap<String, Backend>,
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

    let mut server = Server {
        backends: BTreeMap::new(),
    };

    for (name, plugin) in config.plugins.iter() {
        let mut cmd = Command::new(&plugin.path);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
        cmd.arg("--config")
            .arg(args.config_path.as_os_str())
            .arg("--plugin-name")
            .arg(&name)
            .env("RUST_BACKTRACE", "1");
        println!("spawning {}: {:?}", name, plugin);

        let mut child = cmd.spawn()?;
        let input = child.stdin().take().unwrap();
        let output = child.stdout().take().unwrap();

        let stdout = Framed::<_, Value, Value, _>::new(
            FramedRead::new(output, BytesCodec::new()),
            Json::<Value, Value>::default(),
        );
        tokio::spawn(stdout.for_each(|message| {
            async move {
                println!("json: {:?}", message);
            }
        }));

        let backend = Backend {};
        server.backends.insert(name.clone(), backend);

        tokio::spawn(child);
    }

    // listen for clients
    let mut listener = TcpListener::bind((config.bind_host.as_ref(), config.bind_port)).await?;
    let client_loop = async move {
        loop {
            let (socket, _) = listener.accept().await?;
            println!("ACCEPTED");
        }

        #[allow(unreachable_code)]
        Ok::<_, Error>(())
    };
    tokio::spawn(client_loop);

    // Ok(future::join_all(futures).map(|_| ()).await)
    Ok(future::pending().await)
}
