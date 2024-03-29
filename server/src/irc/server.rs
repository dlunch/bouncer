use std::{collections::HashMap, iter, net::Ipv4Addr, sync::Arc};

use async_trait::async_trait;
use futures::{stream::BoxStream, FutureExt, StreamExt};
use log::{debug, error};
use tokio::{
    io::Result,
    net::TcpListener,
    sync::{
        broadcast::{channel, Sender},
        Mutex,
    },
    task,
};
use tokio_stream::wrappers::{BroadcastStream, TcpListenerStream};

use super::{
    message::{Message as IRCMessage, Prefix as IRCPrefix, Reply as IRCReply},
    transport::Transport,
};
use crate::message::Message;
use crate::sink::Sink;

struct Transports {
    data: HashMap<u32, Transport>,
    index: u32,
}

impl Transports {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            index: 0,
        }
    }

    pub fn insert(&mut self, transport: &Transport) -> u32 {
        let index = self.index;
        self.index += 1;

        self.data.insert(index, transport.clone());

        index
    }

    pub fn remove(&mut self, index: u32) {
        self.data.remove(&index);
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Transport> {
        self.data.iter_mut().map(|x| x.1)
    }
}

struct Context {
    nickname: String,
}

pub struct Server {
    sender: Sender<(IRCMessage, Transport)>,
    streams: Arc<Mutex<Transports>>,
    context: Mutex<Context>,
}

impl Server {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let (sender, _) = channel(16);
        let streams = Arc::new(Mutex::new(Transports::new()));

        let result = Self {
            sender: sender.clone(),
            streams,
            context: Mutex::new(Context { nickname: "".into() }),
        };

        let streams = result.streams.clone();
        task::spawn(async {
            Self::accept_loop(listener, sender, streams).await.unwrap();
        });

        Ok(result)
    }

    async fn accept_loop(listener: TcpListener, sender: Sender<(IRCMessage, Transport)>, transports: Arc<Mutex<Transports>>) -> Result<()> {
        let mut incoming = TcpListenerStream::new(listener);

        while let Some(stream) = incoming.next().await {
            let transport = Transport::new(stream?);
            let sender = sender.clone();

            let transports = transports.clone();
            task::spawn(async move {
                Self::read_loop(transport, sender, transports).await.unwrap();
            });
        }

        Ok(())
    }

    async fn read_loop(transport: Transport, sender: Sender<(IRCMessage, Transport)>, transports: Arc<Mutex<Transports>>) -> Result<()> {
        let index = transports.lock().await.insert(&transport);

        let mut stream = transport.stream().await;
        while let Some(message) = stream.next().await {
            sender.send((message, transport.clone())).unwrap_or(0); // TODO error handling
        }

        transports.lock().await.remove(index);

        Ok(())
    }

    async fn send_response(&self, receiver: &Transport, message: IRCMessage) -> Result<()> {
        debug!("To Client: {}", message);
        receiver.send_message(&message).await
    }

    async fn handle_message(&self, sender: &Transport, message: IRCMessage) -> Result<Option<Message>> {
        debug!("From Client: {}", message);

        Ok(match message.command.as_ref() {
            "USER" => {
                let response = IRCMessage::new(
                    Some(Self::server_prefix()),
                    IRCReply::ERR_NOMOTD,
                    vec!["testtest", "MOTD File is missing"],
                );

                self.send_response(sender, response).await?;

                None
            }
            "CAP" => None,
            "NICK" => {
                self.context.lock().await.nickname = message.args[1].clone();

                None
            }
            "PING" => {
                let response = IRCMessage::new(Some(Self::server_prefix()), "PONG", vec![message.args[0].as_ref()]);

                self.send_response(sender, response).await?;

                None
            }
            "PRIVMSG" => Some(Message::Chat {
                channel: message.args[0].clone(),
                content: message.args[1].clone(),
                sender: message.prefix.as_ref().unwrap().raw().into(),
            }),
            "JOIN" => Some(Message::JoinChannel {
                channel: message.args[0].clone(),
            }),
            _ => {
                error!("Unhandled {}", message.command);

                None
            }
        })
    }

    fn convert_message(&self, message: &Message) -> Vec<IRCMessage> {
        match message {
            Message::Chat { sender, channel, content } => vec![IRCMessage {
                prefix: Some(IRCPrefix::from_raw(sender.into())),
                command: "PRIVMSG".into(),
                args: vec![channel.into(), content.into()],
            }],
            Message::JoinedChannel { channel, sender } => vec![IRCMessage {
                prefix: Some(IRCPrefix::from_raw(sender.into())),
                command: "JOIN".into(),
                args: vec![channel.into()],
            }],
            Message::UsersList { channel, users } => vec![
                IRCMessage {
                    prefix: Some(Self::server_prefix()),
                    command: IRCReply::RPL_NAMREPLY.into(),
                    args: iter::once(channel.into()).chain(users.iter().cloned()).collect::<Vec<_>>(),
                },
                IRCMessage {
                    prefix: Some(Self::server_prefix()),
                    command: IRCReply::RPL_ENDOFNAMES.into(),
                    args: vec![channel.into(), "End of /NAMES list.".into()],
                },
            ],

            _ => unreachable!(),
        }
    }

    fn server_prefix() -> IRCPrefix {
        IRCPrefix::Server("irc.proxy".into())
    }
}

#[async_trait]
impl Sink for Server {
    fn stream(&self) -> BoxStream<Message> {
        BroadcastStream::new(self.sender.subscribe())
            .filter_map(move |x| {
                async move {
                    let (message, sender) = x.unwrap(); // TODO error handling
                    self.handle_message(&sender, message).await.unwrap()
                }
                .boxed()
            })
            .boxed()
    }

    async fn broadcast(&self, message: &Message) -> Result<()> {
        let messages = self.convert_message(message);
        for message in &messages {
            debug!("Broadcast: {}", message);
        }

        let mut streams = self.streams.lock().await;

        for stream in streams.iter_mut() {
            for message in &messages {
                stream.send_message(message).await?;
            }
        }

        Ok(())
    }
}
