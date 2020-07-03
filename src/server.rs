use async_std::io;
use async_std::net::{Ipv4Addr, TcpListener};
use async_std::task;
use async_std::task::JoinHandle;
use futures::StreamExt;

pub trait ServerEventListener: Sync + Send {
    fn on_message<'a>(&self, sender: &'a str, message: &'a str);
}

#[allow(dead_code)]
pub struct Server {
    join_handle: JoinHandle<()>,
}

impl Server {
    pub fn new(port: u16, listener: Box<dyn ServerEventListener>) -> Self {
        let join_handle = task::spawn(async move { Self::start(port, listener).await.unwrap() });

        Self { join_handle }
    }

    #[allow(unused_variables)]
    async fn start(port: u16, listener: Box<dyn ServerEventListener>) -> io::Result<()> {
        let tcp_listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        let mut incoming = tcp_listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let (reader, writer) = &mut (&stream, &stream);
            io::copy(reader, writer).await?;
        }

        Ok(())
    }
}
