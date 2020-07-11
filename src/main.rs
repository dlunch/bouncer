mod bouncer;
mod client;
mod server;

use std::error::Error;

use clap::{App, Arg};

use bouncer::Bouncer;

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

    Bouncer::run(host, port).await;

    Ok(())
}
