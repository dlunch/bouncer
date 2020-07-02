use std::{default::Default, error::Error};

use clap::{App, Arg};
use futures::StreamExt;
use irc::client::prelude::{Client, Command, Config};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let matches = App::new("bouncer")
        .version("1.0")
        .arg(Arg::with_name("host").required(true))
        .arg(Arg::with_name("port").required(true))
        .get_matches();

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap().parse::<u16>().unwrap();

    let config = Config {
        nickname: Some("test".to_owned()),
        server: Some(host.to_owned()),
        port: Some(port),
        channels: vec!["#testtesttest".to_owned()],
        use_tls: Some(false),
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    while let Some(message) = stream.next().await.transpose()? {
        if let Command::PRIVMSG(channel, message) = message.command {
            if message.contains(&*client.current_nickname()) {
                // send_privmsg comes from ClientExt
                client.send_privmsg(&channel, "beep boop").unwrap();
            }
        }
    }

    Ok(())
}
