use std::{collections::HashMap, sync::Arc};

use async_std::{
    io::Result,
    net::{Ipv4Addr, TcpListener},
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use futures::{stream::Stream, StreamExt};
use log::debug;

use super::super::transport::Transport;
use super::super::Message;

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

pub struct ServerImpl {
    receiver: Receiver<(Message, Transport)>,
    streams: Arc<Mutex<Transports>>,
}

impl ServerImpl {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let (sender, receiver) = channel(10);
        let streams = Arc::new(Mutex::new(Transports::new()));

        let streams2 = streams.clone();
        task::spawn(async move {
            Self::accept_loop(listener, sender, streams2).await.unwrap();
        });

        Ok(Self { receiver, streams })
    }

    async fn accept_loop(listener: TcpListener, sender: Sender<(Message, Transport)>, transports: Arc<Mutex<Transports>>) -> Result<()> {
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let transport = Transport::new(stream?);
            let sender = sender.clone();

            let transports2 = transports.clone();
            task::spawn(async move {
                Self::read_loop(transport, sender, transports2).await.unwrap();
            });
        }

        Ok(())
    }

    async fn read_loop(transport: Transport, sender: Sender<(Message, Transport)>, transports: Arc<Mutex<Transports>>) -> Result<()> {
        let index = transports.lock().await.insert(&transport);

        let mut stream = transport.stream();
        while let Some(message) = stream.next().await {
            sender.send((message, transport.clone())).await;
        }

        transports.lock().await.remove(index);

        Ok(())
    }

    pub fn stream(&self) -> impl Stream<Item = (Message, Transport)> {
        self.receiver.clone().map(|(message, sender)| {
            debug!("From Client: {}", message);

            (message, sender)
        })
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        debug!("Broadcast: {}", message);

        let mut streams = self.streams.lock().await;

        for stream in streams.iter_mut() {
            stream.send_message(&message).await?;
        }

        Ok(())
    }

    fn send_response(&self, receiver: Transport, message: Message) -> Result<()> {
        task::spawn(async move { receiver.send_message(&message).await.unwrap() });

        Ok(())
    }

    pub fn handle_message(&self, sender: Transport, message: &Message) -> Result<()> {
        if message.command == "PING" {
            let response = Message::new(None, "PONG", vec![message.args[0].as_ref()]);

            self.send_response(sender, response).unwrap();
        }

        Ok(())
    }
}
