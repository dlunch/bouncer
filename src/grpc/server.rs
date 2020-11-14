use async_std::io::Result;
use async_trait::async_trait;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};

use crate::message::Message;
use crate::sink::Sink;

mod pb {
    tonic::include_proto!("bouncer");
}

pub struct Server {}

#[async_trait]
impl Sink for Server {
    fn stream(&self) -> BoxStream<Message> {
        stream::empty().boxed()
    }

    async fn broadcast(&self, _: &Message) -> Result<()> {
        Ok(())
    }
}
