use std::{collections::HashMap, sync::Arc};

use async_std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{channel, Mutex, Receiver, Sender},
    task,
};
use futures::{
    io,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    stream::Stream,
    StreamExt,
};
use irc::proto::Message;
use log::debug;

type ReadMessage = (String, TcpStream);

struct Streams {
    data: HashMap<u32, TcpStream>,
    index: u32,
}

impl Streams {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            index: 0,
        }
    }

    pub fn insert(&mut self, stream: &TcpStream) -> u32 {
        let index = self.index;
        self.index += 1;

        self.data.insert(index, stream.clone());

        index
    }

    pub fn remove(&mut self, index: u32) {
        self.data.remove(&index);
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TcpStream> {
        self.data.iter_mut().map(|x| x.1)
    }
}

pub struct Server {
    receiver: Receiver<ReadMessage>,
    streams: Arc<Mutex<Streams>>,
}

impl Server {
    pub async fn new(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let (sender, receiver) = channel(10);
        let streams = Arc::new(Mutex::new(Streams::new()));

        let streams2 = streams.clone();
        task::spawn(async move {
            Self::accept_loop(listener, sender, streams2).await.unwrap();
        });

        Ok(Self { receiver, streams })
    }

    async fn accept_loop(listener: TcpListener, sender: Sender<ReadMessage>, streams: Arc<Mutex<Streams>>) -> io::Result<()> {
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let sender = sender.clone();

            let streams2 = streams.clone();
            task::spawn(async move {
                Self::read_loop(stream, sender, streams2).await.unwrap();
            });
        }

        Ok(())
    }

    async fn read_loop(stream: TcpStream, sender: Sender<ReadMessage>, streams: Arc<Mutex<Streams>>) -> io::Result<()> {
        let index = streams.lock().await.insert(&stream);

        let reader = BufReader::new(&stream);
        let mut lines = reader.lines();
        while let Some(line) = lines.next().await {
            sender.send((line?, stream.clone())).await;
        }

        streams.lock().await.remove(index);

        Ok(())
    }

    pub fn stream(&mut self) -> impl Stream<Item = Message> {
        self.receiver.clone().map(|x| {
            let message = x.0.parse::<Message>().unwrap();
            debug!("From Client: {}", message);

            message
        })
    }

    pub async fn send_message(&self, message: Message) -> io::Result<()> {
        debug!("To Clients: {}", message);

        let mut streams = self.streams.lock().await;

        for stream in streams.iter_mut() {
            stream.write(message.to_string().as_bytes()).await?;
        }

        Ok(())
    }
}
