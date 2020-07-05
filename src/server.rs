use std::net::TcpListener;

use async_std::net::{Ipv4Addr, TcpStream};
use futures::{
    io,
    io::{AsyncBufReadExt, BufReader, Lines},
    stream, StreamExt,
};
use irc::proto::Message;

pub struct Server {
    listener: TcpListener,
    streams: Vec<(Lines<BufReader<TcpStream>>, TcpStream)>,
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
            let bufreader = BufReader::new(stream.clone());
            let lines = bufreader.lines();
            self.streams.push((lines, stream));
        }

        let mut stream = stream::select_all(self.streams.iter_mut().map(|x| &mut x.0));
        let result = stream.next().await.unwrap()?;

        Ok(result.parse::<Message>().unwrap())
    }
}
