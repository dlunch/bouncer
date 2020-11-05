use async_std::io::Result;
use futures::{future, select, stream, FutureExt, StreamExt};

use crate::irc::Client;
use crate::irc::Server;
use crate::message::Message;
use crate::sink::Sink;
use crate::source::Source;

pub struct Bouncer {
    source: Box<dyn Source>,
    sinks: Vec<Box<dyn Sink>>,
}

impl Bouncer {
    pub async fn run(host: String, port: u16, server_port: u16) -> Result<()> {
        let client = Box::new(Client::new(host, port).await.unwrap());
        let sinks: Vec<Box<dyn Sink>> = vec![Box::new(Server::new(server_port).await.unwrap())];

        let bouncer = Self { source: client, sinks };

        let mut source_stream = bouncer.source.stream().fuse();
        let mut sinks_stream = stream::select_all(bouncer.sinks.iter().map(|x| x.stream())).fuse();

        loop {
            let res = select! {
                message = source_stream.next() => bouncer.handle_source_message(message.unwrap()).boxed(),
                message = sinks_stream.next() => bouncer.handle_sink_message(message.unwrap()).boxed(),
            };

            res.await?;
        }
    }

    async fn handle_source_message(&self, message: Message) -> Result<()> {
        let futures = self.sinks.iter().map(|x| x.broadcast(&message));

        future::try_join_all(futures).await?;

        Ok(())
    }

    async fn handle_sink_message(&self, message: Message) -> Result<()> {
        self.source.send_message(&message).await
    }
}
