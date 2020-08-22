mod client_impl;

use std::sync::Arc;

use async_std::io::Result;
use futures::{Stream, StreamExt};

use super::Message;

pub struct Client {
    client: Arc<client_impl::ClientImpl>,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let client = Arc::new(client_impl::ClientImpl::new(host, port).await?);

        Ok(Self { client })
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let client = self.client.clone();

        self.client.stream().map(move |message: Message| {
            client.handle_message(&message).unwrap();

            message
        })
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        self.client.send_message(message)
    }
}
