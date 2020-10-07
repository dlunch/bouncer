mod bouncer;
mod irc;
mod message;

use std::error::Error;

use clap::{App, Arg};

use bouncer::Bouncer;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::formatted_timed_builder()
        .filter(Some("bouncer"), log::LevelFilter::Debug)
        .init();

    let matches = App::new("bouncer")
        .version("1.0")
        .arg(Arg::with_name("host").required(true))
        .arg(Arg::with_name("port").required(true))
        .arg(Arg::with_name("server_port").required(true))
        .get_matches();

    let host = matches.value_of("host").unwrap().to_owned();
    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();
    let server_port = matches.value_of("server_port").unwrap().parse::<u16>().unwrap();

    Bouncer::run(host, port, server_port).await?;

    Ok(())
}
