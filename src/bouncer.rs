use async_std::{sync::channel, task};
use futures::{join, select, StreamExt};

use crate::irc::Client;
use crate::irc::Server;

enum Message {
    ServerMessage(irc_proto::Message),
    ClientMessage(irc_proto::Message),
}

pub struct Bouncer {}

impl Bouncer {
    pub async fn run(host: String, port: u16) {
        let (server_sender, server_receiver) = channel(10);
        let (client_sender, client_receiver) = channel(10);

        let client_join_handle = task::spawn(async move {
            let client = Client::new(host, port).await.unwrap();
            let mut client_stream = client.stream().unwrap().fuse();
            let mut server_receiver = server_receiver.fuse();

            loop {
                let res = select! {
                    message = client_stream.next() => Message::ClientMessage(message.unwrap().unwrap()),
                    message = server_receiver.next() => Message::ServerMessage(message.unwrap()),
                };

                match res {
                    Message::ServerMessage(message) => client.send_message(message).await.unwrap(),
                    Message::ClientMessage(message) => client_sender.send(message).await,
                }
            }
        });

        let server_join_handle = task::spawn(async move {
            let mut server = Server::new(6667).await.unwrap();
            let mut server_stream = server.stream().fuse();
            let mut client_receiver = client_receiver.fuse();

            loop {
                let res = select! {
                    message = server_stream.next() => Message::ServerMessage(message.unwrap()),
                    message = client_receiver.next() => Message::ClientMessage(message.unwrap()),
                };

                match res {
                    Message::ServerMessage(message) => server_sender.send(message).await,
                    Message::ClientMessage(message) => server.broadcast(message).await.unwrap(),
                }
            }
        });

        join!(client_join_handle, server_join_handle);
    }
}
