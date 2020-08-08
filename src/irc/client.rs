use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
};
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, Stream, StreamExt};
use irc_proto::{Command, Message};
use log::debug;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();

        let stream = TcpStream::connect(addr).await?;
        let mut result = Self { stream };

        result.send(Command::USER("test".to_owned(), "0".to_owned(), "test".to_owned())).await?;
        result.send(Command::NICK("testtest".to_owned())).await?;

        Ok(result)
    }

    pub fn stream(&self) -> Result<impl Stream<Item = Result<Message>>> {
        let reader = BufReader::new(self.stream.clone());

        Ok(reader.lines().map(|x| {
            let message = x?.parse::<Message>().unwrap();
            debug!("From Origin: {}", message);

            Ok(message)
        }))
    }

    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        debug!("To Origin: {}", message);
        self.stream.write(message.to_string().as_bytes()).await?;

        Ok(())
    }

    async fn send(&mut self, command: Command) -> Result<usize> {
        let message = Message::from(command);

        self.stream.write(message.to_string().as_bytes()).await
    }
}
