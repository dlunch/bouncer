use std::default::Default;

use async_std::task;
use async_std::task::JoinHandle;
use futures::StreamExt;
use irc::{
    client::{data::config::Config, Client as IRCClient},
    error::Result,
    proto::Command,
};

pub trait ClientEventListener: Sync + Send {
    fn on_message<'a>(&self, sender: &'a str, message: &'a str);
}

#[allow(dead_code)]
pub struct Client {
    join_handle: JoinHandle<()>,
}

impl Client {
    pub async fn new(host: String, port: u16, listener: Box<dyn ClientEventListener>) -> Self {
        let config = Config {
            nickname: Some("test".to_owned()),
            server: Some(host),
            port: Some(port),
            channels: vec!["#testtesttest".to_owned()],
            use_tls: Some(false),
            ..Config::default()
        };

        let join_handle = task::spawn(async move {
            Self::start(config, listener).await.unwrap();
        });

        Self { join_handle }
    }

    #[allow(unused_variables)]
    async fn start(config: Config, listener: Box<dyn ClientEventListener>) -> Result<()> {
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

        Ok(())
    }
}
