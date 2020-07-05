use std::default::Default;

use futures::StreamExt;
use irc::{
    client::{data::config::Config, Client as IRCClient, ClientStream},
    error::Result,
    proto::Message,
};

pub struct Client {
    client: IRCClient,
    stream: ClientStream,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let config = Config {
            nickname: Some("test".to_owned()),
            server: Some(host),
            port: Some(port),
            channels: vec!["#testtesttest".to_owned()],
            use_tls: Some(false),
            ..Config::default()
        };

        let mut client = IRCClient::from_config(config).await?;
        client.identify()?;
        let stream = client.stream()?;

        Ok(Self { client, stream })
    }

    pub async fn next_message(&mut self) -> Result<Message> {
        self.stream.next().await.unwrap()
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        self.client.send(message)
    }
}
