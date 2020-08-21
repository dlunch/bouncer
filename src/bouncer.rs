use std::sync::Arc;

use async_std::{
    sync::{channel, Sender},
    task,
};
use futures::{join, select, StreamExt};
use irc_proto::{Command, Message, Prefix, Response};

use crate::irc::Client;
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
        if let Some(Prefix::ServerName(_)) = message.prefix {
            message = Self::create_response_message(message.command);
        }
        self.source_to_sink.send(message).await
    }

    async fn handle_sink_message(&self, message: Message) {
        match &message.command {
            Command::USER(_, _, _) => {
                // send end of motd
                let response = Self::create_response_message(Command::Response(
                    Response::ERR_NOMOTD,
                    vec!["testtest".to_owned(), "MOTD File is missing".to_owned()],
                ));
                self.source_to_sink.send(response).await;
                return;
            }
            Command::CAP(_, _, _, _) => {
                return;
            }
            Command::NICK(_) => {
                return;
            }
            Command::PING(x, _) => {
                Self::create_response_message(Command::PONG(x.to_owned(), None));
            }
            _ => {}
        };
        self.sink_to_source.send(message).await
    }

    fn create_response_message(command: Command) -> Message {
        Message {
            tags: None,
            prefix: Some(Prefix::ServerName("irc-proxy".to_owned())),
            command,
        }
    }
}
