use async_std::io::Result;
use futures::{future, select, stream, FutureExt, StreamExt};

use crate::client::Client;
use crate::irc::IRCClient;
use crate::irc::IRCServer;
use crate::message::Message;
use crate::server::Server;

pub struct Bouncer {
    client: Box<dyn Client>,
    servers: Vec<Box<dyn Server>>,
}

impl Bouncer {
    pub async fn run(host: String, port: u16, server_port: u16) -> Result<()> {
        let client = Box::new(IRCClient::new(host, port).await.unwrap());
        let servers: Vec<Box<dyn Server>> = vec![Box::new(IRCServer::new(server_port).await.unwrap())];

        let bouncer = Self { client, servers };

        let mut client_stream = bouncer.client.stream().fuse();
        let mut server_stream = stream::select_all(bouncer.servers.iter().map(|x| x.stream())).fuse();

        loop {
            let res = select! {
                message = client_stream.next() => bouncer.handle_client_message(message.unwrap()).boxed(),
                message = server_stream.next() => bouncer.handle_server_message(message.unwrap()).boxed(),
            };

            res.await?;
        }
    }

    async fn handle_client_message(&self, message: Message) -> Result<()> {
        let futures = self.servers.iter().map(|x| x.broadcast(&message));

        future::try_join_all(futures).await?;

        Ok(())
    }

    async fn handle_server_message(&self, message: Message) -> Result<()> {
        self.client.send_message(&message).await
    }
}
