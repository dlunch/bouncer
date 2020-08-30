use std::sync::Arc;

use async_std::{
    sync::{channel, Sender},
    task,
};
use futures::{join, select, StreamExt};

use crate::irc::Client;
use crate::irc::Message;
use crate::irc::Server;

enum BouncerMessage {
    SinkMessage(Message),
    SourceMessage(Message),
}

pub struct Bouncer {
    sink_to_source: Sender<Message>,
    source_to_sink: Sender<Message>,
}

impl Bouncer {
    pub async fn run(host: String, port: u16) {
        let (sink_to_source, sink_from_source) = channel(10);
        let (source_to_sink, source_from_sink) = channel(10);

        let bouncer = Arc::new(Self {
            sink_to_source,
            source_to_sink,
        });

        let bouncer_clone = bouncer.clone();
        let source_join_handle = task::spawn(async move {
            let source = Client::new(host, port).await.unwrap();
            let mut source_stream = source.stream().fuse();
            let mut sink_from_source = sink_from_source.fuse();

            loop {
                let res = select! {
                    message = source_stream.next() => BouncerMessage::SourceMessage(message.unwrap()),
                    message = sink_from_source.next() => BouncerMessage::SinkMessage(message.unwrap()),
                };

                match res {
                    BouncerMessage::SinkMessage(message) => source.send_message(message).unwrap(),
                    BouncerMessage::SourceMessage(message) => bouncer_clone.handle_source_message(message).await,
                }
            }
        });

        let bouncer_clone = bouncer.clone();
        let sink_join_handle = task::spawn(async move {
            let sink = Server::new(16667).await.unwrap();
            let mut sink_stream = sink.stream().fuse();
            let mut source_from_sink = source_from_sink.fuse();

            loop {
                let res = select! {
                    message = sink_stream.next() => BouncerMessage::SinkMessage(message.unwrap()),
                    message = source_from_sink.next() => BouncerMessage::SourceMessage(message.unwrap()),
                };

                match res {
                    BouncerMessage::SinkMessage(message) => bouncer_clone.handle_sink_message(message).await,
                    BouncerMessage::SourceMessage(message) => sink.broadcast(message).await.unwrap(),
                }
            }
        });

        join!(source_join_handle, sink_join_handle);
    }

    async fn handle_source_message(&self, mut message: Message) {
        if let Some(x) = message.prefix {
            let prefix = if !(x.contains('!') && x.contains('@')) {
                // TODO temp server detection
                "irc-proxy"
            } else {
                &x
            };

            message = Self::create_response_message(
                Some(prefix),
                &message.command,
                message.args.iter().map(|x| x.as_ref()).collect::<Vec<_>>(),
            );
        }
        self.source_to_sink.send(message).await
    }

    async fn handle_sink_message(&self, message: Message) {
        match message.command.as_ref() {
            "USER" => {
                // ERR_NOMOTD
                let response = Self::create_response_message(Some("irc-proxy"), "422", vec!["testtest", "MOTD File is missing"]);
                self.source_to_sink.send(response).await;
                return;
            }
            "CAP" => {
                return;
            }
            "NICK" => {
                return;
            }
            "PING" => {
                Self::create_response_message(Some("irc-proxy"), "PONG", vec![message.args[0].as_ref()]);
            }
            _ => {}
        };
        self.sink_to_source.send(message).await
    }

    fn create_response_message(prefix: Option<&str>, command: &str, args: Vec<&str>) -> Message {
        Message::new(prefix, command, args)
    }
}
