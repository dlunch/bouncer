use std::ops::Drop;

use async_std::{
    io,
    io::BufReader,
    net::{Ipv4Addr, TcpListener, TcpStream},
    task,
    task::JoinHandle,
};
use futures::{
    channel::{mpsc, mpsc::UnboundedReceiver},
    future::select_all,
    io::AsyncBufReadExt,
    poll,
    task::Poll,
    FutureExt, StreamExt,
};
use irc::proto::Message;

#[allow(dead_code)]
pub struct Server {
    accept_receiver: UnboundedReceiver<TcpStream>,
    accepter_join_handle: Option<JoinHandle<()>>,
    streams: Vec<(BufReader<TcpStream>, TcpStream)>,
}

impl Server {
    pub async fn new(port: u16) -> io::Result<Self> {
        let (sender, accept_receiver) = mpsc::unbounded();

        let listener = TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
        let join_handle = task::spawn(async move {
            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                let stream = stream.unwrap();
                let result = sender.unbounded_send(stream);

                if result.is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            accept_receiver,
            accepter_join_handle: Some(join_handle),
            streams: Vec::new(),
        })
    }

    pub async fn next_message(&mut self) -> io::Result<Message> {
        let accept_future = self.accept_receiver.next();

        #[allow(unused_variables)]
        if let Poll::Ready(x) = poll!(accept_future) {
            let stream = x.unwrap();
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

impl Drop for Server {
    fn drop(&mut self) {
        let join_handle = self.accepter_join_handle.take().unwrap();
        task::spawn(async { join_handle.cancel().await });
    }
}
