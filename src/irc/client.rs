use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
};
use async_trait::async_trait;
use futures::{stream::BoxStream, FutureExt, StreamExt};
use log::{debug, error};

use super::{
    message::{Message as IRCMessage, Reply as IRCReply},
    transport::Transport,
};
use crate::client::Client;
use crate::message::Message;

pub struct IRCClient {
    transport: Transport,
}

impl IRCClient {
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
            IRCReply::RPL_ENDOFMOTD | IRCReply::ERR_NOMOTD => {
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
            IRCReply::RPL_NAMREPLY => {
                if let [_client, _symbol, channel, items] = message.args.as_slice() {
                    Some(Message::NamesList {
                        channel: channel.clone(),
                        users: items
                            .split(' ')
                            .filter_map(|x| if !x.is_empty() { Some(x.to_owned()) } else { None })
                            .collect::<Vec<_>>(),
                    })
                } else {
                    panic!()
                }
            }
            IRCReply::RPL_ENDOFNAMES => Some(Message::NamesEnd {
                channel: message.args[1].clone(),
            }),
            _ => {
                error!("Unhandled {}", message.command);

                None
            }
        })
    }

    fn convert_message(&self, message: &Message) -> IRCMessage {
        match message {
            Message::Chat { channel, content, .. } => IRCMessage {
                prefix: None,
                command: "PRIVMSG".into(),
                args: vec![channel.into(), content.into()],
            },
            Message::JoinChannel { channel } => IRCMessage {
                prefix: None,
                command: "JOIN".into(),
                args: vec![channel.into()],
            },
            _ => unreachable!(),
        }
    }
}

#[async_trait]
impl Client for IRCClient {
    fn stream(&self) -> BoxStream<Message> {
        self.transport
            .stream()
            .filter_map(move |message| async move { self.handle_message(&message).await.unwrap() }.boxed())
            .boxed()
    }

    async fn send_message(&self, message: &Message) -> Result<()> {
        let message = self.convert_message(message);
        debug!("To Origin: {}", message);

        self.transport.send_message(&message).await?;

        Ok(())
    }
}
