use futures::future::{self, FutureExt};
use futures::stream::StreamExt;
use irc_async::{Client, ClientError, Config};

type Result<T> = std::result::Result<T, ClientError>;

async fn run() -> Result<()> {
    let config = Config {
        host: "127.0.0.1".into(),
        port: 4444,
        ssl: false,
        nick: "hello".into(),
    };
    let (mut client, fut) = Client::with_config(config).await?;
    client.register().await?;

    let handler = async {
        while let Some(Ok(message)) = client.next().await {
            println!("message: {:?}", message);
            client.send(message).await.unwrap();
        }
    };

    future::join(fut, handler).map(|(first, _)| first).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("err: {:?}", err);
    }
}
