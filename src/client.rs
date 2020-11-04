use std::boxed::Box;

use async_std::io::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::message::Message;

#[async_trait]
pub trait Client: Sync + Send {
    fn stream(&self) -> BoxStream<Message>;
    async fn send_message(&self, message: &Message) -> Result<()>;
}
