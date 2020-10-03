use async_std::io::Result;
use futures::{select, FutureExt, StreamExt};

use crate::irc::Client;
use crate::irc::Message;
use crate::irc::Server;

pub struct Bouncer {
    client: Client,
    server: Server,
}

impl Bouncer {
    pub async fn run(host: String, port: u16) -> Result<()> {
        let client = Client::new(host, port).await.unwrap();
        let server = Server::new(16667).await.unwrap();

        let bouncer = Self { client, server };

        let mut client_stream = bouncer.client.stream().fuse();
        let mut server_stream = bouncer.server.stream().fuse();

        loop {
            let res = select! {
                message = client_stream.next() => bouncer.handle_client_message(message.unwrap()).boxed(),
                message = server_stream.next() => bouncer.handle_server_message(message.unwrap()).boxed(),
            };

            res.await?;
        }
    }

    async fn handle_client_message(&self, message: Message) -> Result<()> {
        self.server.broadcast(message).await
    }

    async fn handle_server_message(&self, message: Message) -> Result<()> {
        self.client.send_message(message).await
    }
}
