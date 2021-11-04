use std::sync::Arc;

use futures::StreamExt;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Result},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};
use tokio_stream::{wrappers::LinesStream, Stream};

use super::message::Message;

// TODO remove clone
#[derive(Clone)]
pub struct Transport {
    read: Arc<Mutex<Option<OwnedReadHalf>>>,
    write: Arc<Mutex<OwnedWriteHalf>>,
}

impl Transport {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();

        Self {
            read: Arc::new(Mutex::new(Option::Some(read))),
            write: Arc::new(Mutex::new(write)),
        }
    }

    pub async fn stream(&self) -> impl Stream<Item = Message> + '_ {
        let read = self.read.lock().await.take().unwrap();

        LinesStream::new(BufReader::new(read).lines()).map(move |x| Message::from_raw(x.unwrap()))
    }

    pub async fn send_message(&self, message: &Message) -> Result<()> {
        let mut write = self.write.lock().await;
        write.write(message.raw().as_bytes()).await?;

        Ok(())
    }
}
