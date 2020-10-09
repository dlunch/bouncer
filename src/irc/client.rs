use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
};
use futures::{FutureExt, Stream, StreamExt};
use log::debug;

use super::{message::Message as IRCMessage, transport::Transport};
use crate::message::Message;

pub struct Client {
    transport: Transport,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();
        let stream = TcpStream::connect(addr).await?;

        let transport = Transport::new(stream);
        let result = Self { transport };

        result
            .transport
            .send_message(&IRCMessage::new(None, "USER", vec!["test", "0", "*", "test"]))
            .await?;
        result.transport.send_message(&IRCMessage::new(None, "NICK", vec!["testtest"])).await?;

        Ok(result)
    }

    pub fn stream<'a>(&'a self) -> impl Stream<Item = Message> + 'a {
        self.transport
            .stream()
            .filter_map(move |message| async move { self.handle_message(&message).await.unwrap() }.boxed())
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        debug!("To Origin: {:?}", message);

        self.transport.send_message(&self.convert_message(message)).await?;

        Ok(())
    }

    fn on_connected(&self) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&self, message: &IRCMessage) -> Result<Option<Message>> {
        debug!("From Origin: {}", message);

        Ok(match message.command.as_ref() {
            "PING" => {
                let response = IRCMessage::new(None, "PONG", vec![message.args[0].as_ref()]);

                self.transport.send_message(&response).await?;

                None
            }
            "376" | "422" => {
                // RPL_ENDOFMOTD | ERR_NOMOTD
                self.on_connected()?;

                None
            }
            "PRIVMSG" => Some(Message::Chat {
                channel: message.args[0].clone(),
                content: message.args[1].clone(),
                sender: message.prefix.as_ref().unwrap().raw().into(),
            }),
            "JOIN" => Some(Message::JoinedChannel {
                channel: message.args[0].clone(),
                sender: message.prefix.as_ref().unwrap().raw().into(),
            }),
            _ => None,
        })
    }

    fn convert_message(&self, message: Message) -> IRCMessage {
        match message {
            Message::Chat { channel, content, .. } => IRCMessage {
                prefix: None,
                command: "PRIVMSG".into(),
                args: vec![channel, content],
            },
            Message::JoinChannel { channel } => IRCMessage {
                prefix: None,
                command: "JOIN".into(),
                args: vec![channel],
            },
            _ => unreachable!(),
        }
    }
}
