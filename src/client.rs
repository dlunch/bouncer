use std::default::Default;

use futures::StreamExt;
use irc::{
    client::{data::config::Config, Client as IRCClient},
    error::Result,
    proto::Command,
};

pub struct Client {}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let config = Config {
            nickname: Some("test".to_owned()),
            server: Some(host.to_owned()),
            port: Some(port),
            channels: vec!["#testtesttest".to_owned()],
            use_tls: Some(false),
            ..Config::default()
        };

        let mut client = IRCClient::from_config(config).await?;
        client.identify()?;

        let mut stream = client.stream()?;
        while let Some(message) = stream.next().await.transpose()? {
            if let Command::PRIVMSG(channel, message) = message.command {
                if message.contains(&*client.current_nickname()) {
                    // send_privmsg comes from ClientExt
                    client.send_privmsg(&channel, "beep boop").unwrap();
                }
            }
        }

        Ok(Self {})
    }
}
