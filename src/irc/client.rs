use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
};
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, Stream, StreamExt};
use log::debug;

use super::message::Message;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();

        let stream = TcpStream::connect(addr).await?;

        Ok(Self { stream })
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        let reader = BufReader::new(self.stream.clone());

        Ok(reader.lines().map(|x| {
            let message = Message::new(x?);
            debug!("From Origin: {}", message);

            Ok(message)
        }))
    }

    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        debug!("To Origin: {}", message);
        self.stream.write(message.raw()).await?;

        Ok(())
    }
}
