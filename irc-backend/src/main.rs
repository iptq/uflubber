#[macro_use]
extern crate anyhow;

mod config;

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use irc_async::{
    proto::{Command, Message as IrcMessage, Response},
    Client, Config as IrcConfig,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use proto::backend::{
    Message, MessageContent, MessageID, Request, RequestBody, Room, RoomID, RoomIDOrUserID, Update,
    UserID,
};
use serde_json::Value as JsonValue;
use structopt::StructOpt;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use toml::Value as TomlValue;
use uuid::Uuid;

use crate::config::Config;

lazy_static! {
    static ref JOIN_REQUESTS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
}

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "config", parse(from_os_str))]
    config_path: PathBuf,

    #[structopt(long = "backend-name")]
    backend_name: String,
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

    let backend_config = config
        .get("backends")
        .ok_or_else(|| anyhow!("No 'backends' under config.toml"))?
        .get(&args.backend_name)
        .ok_or_else(|| anyhow!("Backend name '{}' not found", args.backend_name))?
        .get("config")
        .ok_or_else(|| anyhow!("Backend has no config"))?
        .clone()
        .try_into::<Config>()?;
    let irc_config = IrcConfig::from(&backend_config);

    let (mut client, fut, client_tx) = Client::with_config(irc_config).await?;
    client.register().await?;
    tokio::spawn(fut);

    let mut stdin = Framed::<_, Request, (), _>::new(
        FramedRead::new(io::stdin(), BytesCodec::new()),
        Json::<Request, ()>::default(),
    );
    let stdin_loop = async move {
        let mut client_tx = client_tx.clone();
        while let Some(Ok(request)) = stdin.next().await {
            match request.body {
                RequestBody::RoomJoin(room_id) => {
                    JOIN_REQUESTS.lock().insert(room_id.0.clone());
                    client_tx
                        .send(IrcMessage {
                            tags: None,
                            prefix: None,
                            command: Command::JOIN(room_id.0, None, None),
                        })
                        .await;
                }
                _ => (),
            }
        }
    };
    tokio::spawn(stdin_loop);

    // main loop
    let mut stdout = Framed::<_, JsonValue, JsonValue, _>::new(
        FramedWrite::new(io::stdout(), BytesCodec::new()),
        Json::<JsonValue, JsonValue>::default(),
    );
    async {
        while let Some(Ok(message)) = client.next().await {
            let flubber_message = match message.command {
                Command::PRIVMSG(target, content) => {
                    let recipient = if target.starts_with('&') || target.starts_with('#') {
                        RoomIDOrUserID::Room(RoomID(target))
                    } else {
                        RoomIDOrUserID::User(UserID(target))
                    };
                    let new_message = Message {
                        attachments: Vec::new(),
                        content: MessageContent::Text(content),
                        create_time: Utc::now(),
                        edit_time: Utc::now(),
                        extra: JsonValue::Null,
                        id: MessageID(Uuid::new_v4().to_string()),
                        sender: UserID(message.prefix.unwrap()),
                        recipient,
                    };
                    Some(Update::MessageUpsert(new_message))
                }
                Command::Response(response, mut args, last_arg) => {
                    eprintln!("response: {:?} {:?} {:?}", response, args, last_arg);
                    if let Some(arg) = last_arg {
                        args.push(arg);
                    }
                    match response {
                        Response::RPL_TOPIC => {
                            let mut args = args.into_iter();
                            // check if the channel we joined is actually in the list of channels we requested
                            if let Some(channel_name) = args.next() {
                                let mut join_requests = JOIN_REQUESTS.lock();
                                if join_requests.contains(&channel_name) {
                                    join_requests.remove(&channel_name);
                                    let new_room = Room {
                                        id: RoomID(channel_name.to_string()),
                                        name: channel_name.to_string(),
                                        parent: None,
                                        sendable: true, // todo
                                    };
                                    Some(Update::RoomUpsert(new_room))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some(message) = flubber_message {
                stdout.send(serde_json::to_value(message).unwrap()).await;
            }
        }
    }
        .await;

    Ok(())
}
