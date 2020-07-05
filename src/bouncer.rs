use futures::{select, FutureExt};
use irc::proto::Message;

use crate::client::Client;
use crate::server::Server;

pub struct Bouncer {
    client: Client,
    server: Server,
}

impl Bouncer {
    pub async fn new(host: String, port: u16) -> Self {
        let client = Client::new(host, port).await.unwrap();
        let server = Server::new(6667).await.unwrap();

        Self { client, server }
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                client_message = self.client.next_message().fuse() => self.handle_client_message(client_message.unwrap()).await,
                server_message = self.server.next_message().fuse() => self.handle_server_message(server_message.unwrap()).await,
            };
        }
    }

    async fn handle_client_message(&self, message: Message) {
        println!("{}", message);
        self.client.send_message(message).unwrap();
    }

    async fn handle_server_message(&self, message: Message) {
        println!("{}", message);
    }
}
