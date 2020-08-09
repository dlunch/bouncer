use std::sync::Arc;

use async_std::io::Result;
use futures::{stream::Stream, StreamExt};
use irc_proto::Message;

mod server_impl {
    use std::{collections::HashMap, sync::Arc};

    use async_std::{
        io::Result,
        net::{Ipv4Addr, TcpListener, TcpStream},
        sync::{channel, Mutex, Receiver, Sender},
        task,
    };
    use futures::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        stream::Stream,
        StreamExt,
    };
    use irc_proto::{Command, Message};
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

    pub struct ServerImpl {
        receiver: Receiver<ReadMessage>,
        streams: Arc<Mutex<Streams>>,
    }

    impl ServerImpl {
        pub async fn new(port: u16) -> Result<Self> {
            let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

            let (sender, receiver) = channel(10);
            let streams = Arc::new(Mutex::new(Streams::new()));

            let streams2 = streams.clone();
            task::spawn(async move {
                Self::accept_loop(listener, sender, streams2).await.unwrap();
            });

            Ok(Self { receiver, streams })
        }

        async fn accept_loop(listener: TcpListener, sender: Sender<ReadMessage>, streams: Arc<Mutex<Streams>>) -> Result<()> {
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

        async fn read_loop(stream: TcpStream, sender: Sender<ReadMessage>, streams: Arc<Mutex<Streams>>) -> Result<()> {
            let index = streams.lock().await.insert(&stream);

            let reader = BufReader::new(&stream);
            let mut lines = reader.lines();
            while let Some(line) = lines.next().await {
                sender.send((line?, stream.clone())).await;
            }

            streams.lock().await.remove(index);

            Ok(())
        }

        pub fn stream(&self) -> impl Stream<Item = (Message, TcpStream)> {
            self.receiver.clone().map(|(raw, sender)| {
                let message = raw.parse::<Message>().unwrap();
                debug!("From Client: {}", message);

                (message, sender)
            })
        }

        pub async fn broadcast(&self, message: Message) -> Result<()> {
            debug!("Broadcast: {}", message);

            let mut streams = self.streams.lock().await;

            for stream in streams.iter_mut() {
                stream.write(message.to_string().as_bytes()).await?;
            }

            Ok(())
        }

        fn send_response(&self, mut receiver: TcpStream, message: Message) -> Result<()> {
            task::spawn(async move { receiver.write(message.to_string().as_bytes()).await.unwrap() });

            Ok(())
        }

        pub fn handle_message(&self, sender: TcpStream, message: &Message) -> Result<()> {
            if let Command::PING(server1, server2) = &message.command {
                let response = Message::from(Command::PONG(server1.clone(), server2.clone()));

                self.send_response(sender, response).unwrap();
            }

            Ok(())
        }
    }
}

pub struct Server {
    server: Arc<server_impl::ServerImpl>,
}

impl Server {
    pub async fn new(port: u16) -> Result<Self> {
        let server = Arc::new(server_impl::ServerImpl::new(port).await?);

        Ok(Self { server })
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let server_clone = self.server.clone();

        self.server.stream().map(move |(message, sender)| {
            server_clone.handle_message(sender, &message).unwrap();

            message
        })
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        self.server.broadcast(message).await
    }
}
