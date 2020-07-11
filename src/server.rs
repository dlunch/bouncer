use async_std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{channel, Receiver, Sender},
    task,
};
use futures::{
    io,
    io::{AsyncBufReadExt, BufReader},
    StreamExt,
};
use irc::proto::Message;

type ReadMessage = (String, TcpStream);

pub struct Server {
    receiver: Receiver<ReadMessage>,
}

impl Server {
    pub async fn new(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let (sender, receiver) = channel(10);
        task::spawn(async move {
            Self::accept_loop(listener, sender).await.unwrap();
        });

        Ok(Self { receiver })
    }

    async fn accept_loop(listener: TcpListener, sender: Sender<ReadMessage>) -> io::Result<()> {
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let sender = sender.clone();
            task::spawn(async move {
                Self::read_loop(stream, sender).await.unwrap();
            });
        }

        Ok(())
    }

    async fn read_loop(stream: TcpStream, sender: Sender<ReadMessage>) -> io::Result<()> {
        let reader = BufReader::new(&stream);
        let mut lines = reader.lines();
        while let Some(line) = lines.next().await {
            sender.send((line?, stream.clone())).await;
        }

        Ok(())
    }

    pub async fn next_message(&mut self) -> Option<Message> {
        if self.receiver.is_empty() {
            None
        } else {
            let message = self.receiver.recv().await.unwrap();

            Some(message.0.parse::<Message>().unwrap())
        }
    }
}
