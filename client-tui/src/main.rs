#[macro_use]
extern crate serde;

mod config;

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;
use serde_json::Value as JsonValue;
use tokio::{fs::File, io::AsyncReadExt, net::TcpStream};
use tokio_util::codec::{FramedWrite, BytesCodec};
use tokio_serde::{Framed, formats::Json};

use crate::config::Config;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,
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

    let mut stream = TcpStream::connect((config.server_host.as_ref(), config.server_port)).await?;
    let (stream, sink) = stream.split();
    let server_tx = Framed::<_, JsonValue, JsonValue, _>::new(
        FramedWrite::new(sink, BytesCodec::new()),
        Json::<JsonValue, JsonValue>::default(),
    );

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
