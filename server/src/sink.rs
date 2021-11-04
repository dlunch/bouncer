use std::boxed::Box;

use tokio::io::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::message::Message;

#[async_trait]
pub trait Sink: Sync + Send {
    fn stream(&self) -> BoxStream<Message>;
    async fn broadcast(&self, message: &Message) -> Result<()>;
}
