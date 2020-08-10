use std::sync::Arc;

use async_std::{
    sync::{channel, Sender},
    task,
};
use futures::{join, select, StreamExt};
use irc_proto::{Command, Message};

use crate::irc::Client;
use crate::irc::Server;

enum BouncerMessage {
    SinkMessage(Message),
    SourceMessage(Message),
}

pub struct Bouncer {
    sink_sender: Sender<Message>,
    source_sender: Sender<Message>,
}

impl Bouncer {
    pub async fn run(host: String, port: u16) {
        let (sink_sender, sink_receiver) = channel(10);
        let (source_sender, source_receiver) = channel(10);

        let bouncer = Arc::new(Self { sink_sender, source_sender });

        let bouncer_clone = bouncer.clone();
        let source_join_handle = task::spawn(async move {
            let source = Client::new(host, port).await.unwrap();
            let mut source_stream = source.stream().fuse();
            let mut sink_receiver = sink_receiver.fuse();

            loop {
                let res = select! {
                    message = source_stream.next() => BouncerMessage::SourceMessage(message.unwrap()),
                    message = sink_receiver.next() => BouncerMessage::SinkMessage(message.unwrap()),
                };

                match res {
                    BouncerMessage::SinkMessage(message) => source.send_message(message).unwrap(),
                    BouncerMessage::SourceMessage(message) => bouncer_clone.handle_source_message(message).await,
                }
            }
        });

        let bouncer_clone = bouncer.clone();
        let sink_join_handle = task::spawn(async move {
            let sink = Server::new(6667).await.unwrap();
            let mut sink_stream = sink.stream().fuse();
            let mut source_receiver = source_receiver.fuse();

            loop {
                let res = select! {
                    message = sink_stream.next() => BouncerMessage::SinkMessage(message.unwrap()),
                    message = source_receiver.next() => BouncerMessage::SourceMessage(message.unwrap()),
                };

                match res {
                    BouncerMessage::SinkMessage(message) => bouncer_clone.handle_sink_message(message).await,
                    BouncerMessage::SourceMessage(message) => sink.broadcast(message).await.unwrap(),
                }
            }
        });

        join!(source_join_handle, sink_join_handle);
    }

    async fn handle_source_message(&self, message: Message) {
        self.source_sender.send(message).await
    }

    async fn handle_sink_message(&self, message: Message) {
        self.sink_sender.send(message).await
    }
}
