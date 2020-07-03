mod client;
mod server;

use std::error::Error;

use clap::{App, Arg};

use client::Client;
use server::{Server, ServerEventListener};

#[allow(unused_variables)]
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let matches = App::new("bouncer")
        .version("1.0")
        .arg(Arg::with_name("host").required(true))
        .arg(Arg::with_name("port").required(true))
        .get_matches();

    let host = matches.value_of("host").unwrap().to_owned();
    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();

    let client = Client::new(host, port);

    struct EventListener {}
    impl ServerEventListener for EventListener {
        fn on_message<'a>(&self, sender: &'a str, message: &'a str) {}
    }

    let listener = EventListener {};
    let server = Server::new(6667, Box::new(listener));

    Ok(())
}
