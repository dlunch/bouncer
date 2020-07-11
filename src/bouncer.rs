use async_std::{sync::channel, task};
use futures::{join, select, StreamExt};

use crate::client::Client;
use crate::server::Server;

enum Message {
    ServerMessage(irc::proto::Message),
    ClientMessage(irc::proto::Message)
}

pub struct Bouncer {}

impl Bouncer {
    pub async fn run(host: String, port: u16) {
        let (sender, receiver) = channel(10);

        let client_join_handle = task::spawn(async move {
            let mut client = Client::new(host, port).await.unwrap();
            let mut client_stream = client.stream().unwrap().fuse();
            let mut receiver = receiver.fuse();

            loop {
                let res = select! {
                    client_message = client_stream.next() => Message::ClientMessage(client_message.unwrap().unwrap()),
                    server_message = receiver.next() => Message::ServerMessage(server_message.unwrap()),
                };

                if let Message::ServerMessage(message) = res {
                    client.send_message(message).unwrap();
                }
            }
        });

        let server_join_handle = task::spawn(async move {
            let mut server = Server::new(6667).await.unwrap();
            let mut server_stream = server.stream().fuse();
            loop {
                let res = select! {
                    server_message = server_stream.next() => Message::ServerMessage(server_message.unwrap())
                };

                if let Message::ServerMessage(message) = res {
                    sender.send(message).await;
                }
            }
        });

        join!(client_join_handle, server_join_handle);
    }
}
