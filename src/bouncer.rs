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

    async fn handle_client_message(&self, mut message: Message) -> Result<()> {
        if let Some(x) = message.prefix {
            let prefix = if !(x.contains('!') && x.contains('@')) {
                // TODO temp server detection
                "irc-proxy"
            } else {
                &x
            };

            message = Message::new(
                Some(prefix),
                &message.command,
                message.args.iter().map(|x| x.as_ref()).collect::<Vec<_>>(),
            );
        }
        self.server.broadcast(message).await
    }

    async fn handle_server_message(&self, message: Message) -> Result<()> {
        match message.command.as_ref() {
            "USER" => {
                // ERR_NOMOTD
                let message = Message::new(Some("irc-proxy"), "422", vec!["testtest", "MOTD File is missing"]);

                return self.server.broadcast(message).await;
            }
            "CAP" => {
                return Ok(());
            }
            "NICK" => {
                return Ok(());
            }
            "PING" => {
                let message = Message::new(Some("irc-proxy"), "PONG", vec![message.args[0].as_ref()]);

                return self.server.broadcast(message).await;
            }
            _ => {}
        };

        self.client.send_message(message)
    }
}
