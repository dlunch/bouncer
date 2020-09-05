mod client_impl;

use std::sync::Arc;

use async_std::io::Result;
use futures::{FutureExt, Stream, StreamExt};

use super::Message;

pub struct Client {
    client: Arc<client_impl::ClientImpl>,
}

impl Client {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let client = Arc::new(client_impl::ClientImpl::new(host, port).await?);

        client.send_message(Message::new(None, "USER", vec!["test", "0", "*", "test"])).await?;
        client.send_message(Message::new(None, "NICK", vec!["testtest"])).await?;

        Ok(Self { client })
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let client = self.client.clone();

        self.client.stream().then(move |message: Message| {
            let client = client.clone();
            async move {
                client.handle_message(&message).await.unwrap();

                message
            }
            .boxed()
        })
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.client.send_message(message).await
    }
}
