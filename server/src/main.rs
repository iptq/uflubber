#[macro_use]
extern crate serde;

mod client;
mod config;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Error;
use anyhow::Result;
use futures::{
    future::{self},
    stream::StreamExt,
};
use proto::backend::InitInfo;
use rusqlite::Connection;
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use tokio::{self, fs::File, io::AsyncReadExt, net::TcpListener, process::Command};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::client::Client;
use crate::config::Config;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,
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

    let conn = Connection::open_in_memory()?;
    let mut server = Server {
        backends: BTreeMap::new(),
    };

    for (name, backend) in config.backends.iter() {
        let mut cmd = Command::new(&backend.path);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
        cmd.arg("--config")
            .arg(args.config_path.as_os_str())
            .arg("--backend-name")
            .arg(&name)
            .env("RUST_BACKTRACE", "1");
        println!("spawning {}: {:?}", name, cmd);

        let mut child = cmd.spawn()?;
        let input = child.stdin().take().unwrap();
        let output = child.stdout().take().unwrap();

        let mut stdout = Framed::<_, JsonValue, JsonValue, _>::new(
            FramedRead::new(output, BytesCodec::new()),
            Json::<JsonValue, JsonValue>::default(),
        );
        let init: InitInfo = match stdout.next().await {
            Some(Ok(value)) => serde_json::from_value(value).unwrap(),
            _ => {
                eprintln!("invalid backend, did not send init info");
                continue;
            }
        };
        eprintln!("backend init: {:?}", init);
        tokio::spawn(stdout.for_each(|message| {
            async move {
                match message {
                    Ok(message) => println!("json: {}", message),
                    Err(err) => eprintln!("error: {}", err),
                }
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

    Ok(future::pending().await)
}
