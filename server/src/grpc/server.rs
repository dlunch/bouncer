use async_std::{io, net::Ipv4Addr, task::spawn};
use async_trait::async_trait;
use futures::{
    stream::{self, BoxStream},
    StreamExt,
};

use crate::message::Message;
use crate::sink::Sink;

use tonic::{metadata::MetadataValue, transport, Request, Response, Status};

mod pb {
    tonic::include_proto!("bouncer");
}
use pb::{LoginRequest, LoginResponse};

struct GrpcServer {}

#[async_trait]
impl pb::bouncer_server::Bouncer for GrpcServer {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let username = request.into_inner().username;

        Ok(Response::new(LoginResponse { token: username }))
    }
}

pub struct Server {}

impl Server {
    pub fn new(port: u16) -> Self {
        spawn(async move {
            let addr = (Ipv4Addr::new(0, 0, 0, 0), port);

            let server = pb::bouncer_server::BouncerServer::with_interceptor(GrpcServer {}, Server::check_auth);
            transport::Server::builder().add_service(server).serve(addr.into()).await.unwrap();
        });

        Self {}
    }

    fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
        let token = MetadataValue::from_str("Bearer some-secret-token").unwrap();

        match req.metadata().get("authorization") {
            Some(t) if token == t => Ok(req),
            _ => Err(Status::unauthenticated("No valid auth token")),
        }
    }
}

#[async_trait]
impl Sink for Server {
    fn stream(&self) -> BoxStream<Message> {
        stream::empty().boxed()
    }

    async fn broadcast(&self, _: &Message) -> io::Result<()> {
        Ok(())
    }
}
