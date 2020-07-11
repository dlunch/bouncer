use std::default::Default;

use irc::{
    client::{data::config::Config, Client as IRCClient, ClientStream},
    error::Result,
    proto::Message,
};

pub struct Client {
    client: IRCClient,
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

        let client = IRCClient::from_config(config).await?;
        client.identify()?;

        Ok(Self { client })
    }

    pub fn stream(&mut self) -> Result<ClientStream> {
        self.client.stream()
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        self.client.send(message)
    }
}
