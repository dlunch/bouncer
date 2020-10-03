use std::{collections::HashMap, sync::Arc};

use async_std::{
    io::Result,
    net::{Ipv4Addr, TcpListener},
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use futures::{FutureExt, Stream, StreamExt};
use log::debug;

use super::{
    message::{Message as IRCMessage, Prefix as IRCPrefix},
    transport::Transport,
};
use crate::message::Message;

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

pub struct Server {
    receiver: Receiver<(IRCMessage, Transport)>,
    streams: Arc<Mutex<Transports>>,
}

impl Server {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let (sender, receiver) = channel(10);
        let streams = Arc::new(Mutex::new(Transports::new()));

        let result = Self { receiver, streams };

        let streams = result.streams.clone();
        task::spawn(async {
            Self::accept_loop(listener, sender, streams).await.unwrap();
        });

        Ok(result)
    }

    async fn accept_loop(listener: TcpListener, sender: Sender<(IRCMessage, Transport)>, transports: Arc<Mutex<Transports>>) -> Result<()> {
        let mut incoming = listener.incoming();

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

        let mut stream = transport.stream();
        while let Some(message) = stream.next().await {
            sender.send((message, transport.clone())).await;
        }

        transports.lock().await.remove(index);

        Ok(())
    }

    pub fn stream<'a>(&'a self) -> impl Stream<Item = Message> + 'a {
        self.receiver
            .clone()
            .filter_map(move |(message, sender)| async move { self.handle_message(&sender, message).await.unwrap() }.boxed())
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        debug!("Broadcast: {:?}", message);

        let mut streams = self.streams.lock().await;

        let irc_message = IRCMessage::from_message(message);
        for stream in streams.iter_mut() {
            stream.send_message(&irc_message).await?;
        }

        Ok(())
    }

    async fn send_response(&self, receiver: &Transport, message: IRCMessage) -> Result<()> {
        debug!("To Client: {}", message);
        receiver.send_message(&message).await
    }

    pub async fn handle_message(&self, sender: &Transport, message: IRCMessage) -> Result<Option<Message>> {
        debug!("From Client: {}", message);

        match message.command.as_ref() {
            "USER" => {
                // ERR_NOMOTD
                let response = IRCMessage::new(Some(Self::server_prefix()), "422", vec!["testtest", "MOTD File is missing"]);

                self.send_response(&sender, response).await?;

                Ok(None)
            }
            "CAP" => Ok(None),
            "NICK" => Ok(None),
            "PING" => {
                let response = IRCMessage::new(Some(Self::server_prefix()), "PONG", vec![message.args[0].as_ref()]);

                self.send_response(&sender, response).await?;

                Ok(None)
            }
            _ => Ok(Some(message.into_message())),
        }
    }

    fn server_prefix() -> IRCPrefix {
        IRCPrefix::Server("irc.proxy".into())
    }
}
