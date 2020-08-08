use std::sync::Arc;

use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
    task,
};
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, Stream, StreamExt};
use irc_proto::{Command, Message};
use log::debug;

struct Transport {
    stream: TcpStream,
}

impl Transport {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();

        let stream = TcpStream::connect(addr).await?;
        let result = Self { stream };

        result
            .send_message(Message::from(Command::USER("test".to_owned(), "0".to_owned(), "test".to_owned())))
            .await?;
        result.send_message(Message::from(Command::NICK("testtest".to_owned()))).await?;

        Ok(result)
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        let reader = BufReader::new(self.stream.clone());

        Ok(reader.lines().map(move |x| {
            let message = x?.parse::<Message>().unwrap();
            debug!("From Origin: {}", message);

            Ok(message)
        }))
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        debug!("To Origin: {}", message);

        let mut stream = self.stream.clone();
        stream.write(message.to_string().as_bytes()).await?;

        Ok(())
    }
}

pub struct Client {
    transport: Arc<Transport>,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let transport = Transport::new(host, port).await?;

        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        let transport = self.transport.clone();

        Ok(self.transport.stream()?.map(move |x| {
            let transport = transport.clone();

            let message = x?;
            let message_clone = message.clone();
            task::spawn(async move {
                Self::handle_message(transport, message_clone).await.unwrap();
            });

            Ok(message)
        }))
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.transport.send_message(message).await
    }

    async fn handle_message(transport: Arc<Transport>, message: Message) -> Result<()> {
        if let Command::PING(x, y) = &message.command {
            let response = Message::from(Command::PONG(x.clone(), y.clone()));

            transport.send_message(response).await?;
        }

        Ok(())
    }
}
