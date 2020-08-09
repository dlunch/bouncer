mod server_impl;

use std::sync::Arc;

use async_std::io::Result;
use futures::{stream::Stream, StreamExt};
use irc_proto::Message;

pub struct Server {
    server: Arc<server_impl::ServerImpl>,
}

impl Server {
    pub async fn new(port: u16) -> Result<Self> {
        let server = Arc::new(server_impl::ServerImpl::new(port).await?);

        Ok(Self { server })
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let server_clone = self.server.clone();

        self.server.stream().map(move |(message, sender)| {
            server_clone.handle_message(sender, &message).unwrap();

            message
        })
    }

    pub async fn broadcast(&self, message: Message) -> Result<()> {
        self.server.broadcast(message).await
    }
}
