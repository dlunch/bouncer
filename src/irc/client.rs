use std::sync::Arc;

use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
    task,
};
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, Stream, StreamExt};
use irc_proto::{Command, Message, Response};
use log::debug;

struct Transport {
    stream: TcpStream,
}

impl Transport {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();
        let stream = TcpStream::connect(addr).await?;

        Ok(Self { stream })
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

pub struct ClientImpl {
    transport: Arc<Transport>,
}

impl ClientImpl {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let transport = Transport::new(host, port).await?;

        transport
            .send_message(Message::from(Command::USER("test".to_owned(), "0".to_owned(), "test".to_owned())))
            .await?;
        transport.send_message(Message::from(Command::NICK("testtest".to_owned()))).await?;

        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        Ok(self.transport.stream()?)
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        let transport = self.transport.clone();

        task::spawn(async move { transport.send_message(message).await.unwrap() });

        Ok(())
    }

    fn on_connected(&self) -> Result<()> {
        Ok(())
    }

    fn handle_message(&self, message: &Message) -> Result<()> {
        match &message.command {
            Command::PING(x, y) => {
                let response = Message::from(Command::PONG(x.clone(), y.clone()));

                self.send_message(response)?;
            }
            Command::Response(response, _) => match response {
                Response::RPL_ENDOFMOTD | Response::ERR_NOMOTD => self.on_connected()?,
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}

pub struct Client {
    client: Arc<ClientImpl>,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let client = Arc::new(ClientImpl::new(host, port).await?);

        Ok(Self { client })
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        let client = self.client.clone();

        Ok(self.client.stream()?.map(move |x| {
            let message = x?;
            client.handle_message(&message)?;

            Ok(message)
        }))
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        self.client.send_message(message)
    }
}
