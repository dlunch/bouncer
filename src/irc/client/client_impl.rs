use std::sync::Arc;

use async_std::{
    io::Result,
    net::{TcpStream, ToSocketAddrs},
    task,
};
use futures::Stream;
use log::debug;

use super::super::transport::Transport;
use super::super::Message;

pub struct ClientImpl {
    transport: Arc<Transport>,
}

impl ClientImpl {
    pub async fn new(host: String, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port).to_socket_addrs().await?.next().unwrap();
        let stream = TcpStream::connect(addr).await?;

        let transport = Transport::new(stream);

        transport.send_message(&Message::new(None, "USER", vec!["test", "0", "test"])).await?;
        transport.send_message(&Message::new(None, "NICK", vec!["testtest"])).await?;

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
        self.send_message(Message::new(None, "JOIN", vec!["#testtesttest"]))?;

        Ok(())
    }

    pub fn handle_message(&self, message: &Message) -> Result<()> {
        debug!("From Origin: {}", message);

        match message.command.as_ref() {
            "PING" => {
                let response = Message::new(None, "PONG", vec![message.args[0].as_ref()]);

                self.send_message(response)?;
            }
            "376" | "422" => {
                // RPL_ENDOFMOTD | ERR_NOMOTD
                self.on_connected()?
            }
            _ => {}
        }

        Ok(())
    }
}
