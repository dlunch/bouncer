
use async_std::{io, io::BufReader, net::{Ipv4Addr, TcpListener, TcpStream}};
use futures::{io::AsyncBufReadExt, FutureExt, poll, task::Poll, pin_mut, future::select_all};
use irc::proto::Message;

#[allow(dead_code)]
pub struct Server {
    listener: TcpListener,
    streams: Vec<TcpStream>,
}

impl Server {
    pub async fn new(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;

        Ok(Self {
            listener,
            streams: Vec::new(),
        })
    }

    pub async fn next_message(&mut self) -> io::Result<Message> {
        let accept_future = self.listener.accept();
        pin_mut!(accept_future);

        #[allow(unused_variables)]
        if let Poll::Ready(x) = poll!(accept_future) {
            // TODO accept
        }

        let reader = self.streams.iter().map(|x| {
            async move {
                let mut reader = BufReader::new(x);
                let mut result = String::new();
                reader.read_line(&mut result).await.unwrap();

                result
            }
            .boxed()
        });

        let (result, _, _) = select_all(reader).await;

        Ok(result.parse::<Message>().unwrap())
    }
}
