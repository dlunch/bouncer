use std::boxed::Box;

use async_trait::async_trait;
use futures::stream::BoxStream;
use tokio::io::Result;

use crate::message::Message;

#[async_trait]
pub trait Source: Sync + Send {
    async fn stream(&self) -> BoxStream<Message>;
    async fn send_message(&self, message: &Message) -> Result<()>;
}
