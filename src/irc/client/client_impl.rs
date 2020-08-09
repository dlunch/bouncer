use std::sync::Arc;

use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
    task,
};
use futures::Stream;
use irc_proto::{Command, Message, Response};
use log::debug;

use super::super::transport::Transport;

pub struct ClientImpl {
    transport: Arc<Transport>,
}

impl ClientImpl {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();
        let stream = TcpStream::connect(addr).await?;

        let transport = Transport::new(stream);

        transport
            .send_message(&Message::from(Command::USER("test".to_owned(), "0".to_owned(), "test".to_owned())))
            .await?;
        transport.send_message(&Message::from(Command::NICK("testtest".to_owned()))).await?;

        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        self.transport.stream()
    }

    pub fn send_message(&self, message: Message) -> Result<()> {
        debug!("To Origin: {}", message);

        let transport = self.transport.clone();
        task::spawn(async move { transport.send_message(&message).await.unwrap() });

        Ok(())
    }

    fn on_connected(&self) -> Result<()> {
        self.send_message(Message::from(Command::JOIN("#testtesttest".to_owned(), None, None)))?;

        Ok(())
    }

    pub fn handle_message(&self, message: &Message) -> Result<()> {
        debug!("From Origin: {}", message);

        match &message.command {
            Command::PING(x, y) => {
                let response = Message::from(Command::PONG(x.clone(), y.clone()));

                self.send_message(response)?;
            }
            Command::Response(response, _) => match response {
                Response::RPL_ENDOFMOTD | Response::ERR_NOMOTD => self.on_connected()?,
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}
