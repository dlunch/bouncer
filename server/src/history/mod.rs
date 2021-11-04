use async_trait::async_trait;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};
use tokio::io::Result;

use crate::message::Message;
use crate::sink::Sink;

pub struct History {}

impl History {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Sink for History {
    fn stream(&self) -> BoxStream<Message> {
        stream::empty().boxed()
    }

    async fn broadcast(&self, _: &Message) -> Result<()> {
        Ok(())
    }
}
