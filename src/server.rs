use std::net::TcpListener;

use async_std::{
    io,
    io::BufReader,
    net::{Ipv4Addr, TcpStream},
};
use futures::{future::select_all, io::AsyncBufReadExt, FutureExt};
use irc::proto::Message;

pub struct Server {
    listener: TcpListener,
    streams: Vec<(BufReader<TcpStream>, TcpStream)>,
}

impl Server {
    pub async fn new(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port))?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            streams: Vec::new(),
        })
    }

    pub async fn next_message(&mut self) -> io::Result<Message> {
        if let Ok((stream, _)) = self.listener.accept() {
            let stream = TcpStream::from(stream);
            self.streams.push((BufReader::new(stream.clone()), stream));
        }

        let reader = self.streams.iter_mut().map(|x| {
            async move {
                let mut result = String::new();
                x.0.read_line(&mut result).await.unwrap();

                result
            }
            .boxed()
        });

        let (result, _, _) = select_all(reader).await;

        Ok(result.parse::<Message>().unwrap())
    }
}
