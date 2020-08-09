use async_std::{io::Result, net::TcpStream};
use futures::{io::BufReader, AsyncBufReadExt, AsyncWriteExt, Stream, StreamExt};
use irc_proto::Message;

#[derive(Clone)]
pub struct Transport {
    stream: TcpStream,
}

impl Transport {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn stream(&self) -> impl Stream<Item = Result<Message>> {
        let reader = BufReader::new(self.stream.clone());

        reader.lines().map(move |x| {
            let message = x?.parse::<Message>().unwrap();

            Ok(message)
        })
    }

    pub async fn send_message(&self, message: &Message) -> Result<()> {
        let mut stream = self.stream.clone();
        stream.write(message.to_string().as_bytes()).await?;

        Ok(())
    }
}
