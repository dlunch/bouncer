use std::iter;

use crate::message::Message as CommonMessage;

#[derive(Eq, PartialEq)]
pub enum Prefix {
    Server(String),
    User(String),
}

impl Prefix {
    pub fn from_raw(raw: String) -> Self {
        if raw.contains('.') && !(raw.contains('!') && raw.contains('@')) {
            Self::Server(raw)
        } else {
            Self::User(raw)
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            Self::Server(x) => x,
            Self::User(x) => x,
        }
    }
}

pub struct Message {
    pub prefix: Option<Prefix>,
    pub command: String,
    pub args: Vec<String>,
}

impl Message {
    pub fn new(prefix: Option<Prefix>, command: &str, args: Vec<&str>) -> Self {
        let command = command.to_owned();
        let args = args.into_iter().map(|x| x.to_owned()).collect::<Vec<_>>();

        Self { prefix, command, args }
    }

    pub fn from_raw(raw: String) -> Self {
        let mut split = raw.trim_matches(|x: char| x.is_control()).split(' ').peekable();

        let prefix = if split.peek().unwrap().starts_with(':') {
            Some(Prefix::from_raw(split.next().unwrap()[1..].into()))
        } else {
            None
        };

        let command = split.next().unwrap().into();

        let mut args = Vec::<String>::with_capacity(split.size_hint().0);
        while let Some(item) = split.next() {
            if let Some(x) = item.strip_prefix(':') {
                args.push(iter::once(x).chain(split).collect::<Vec<_>>().join(" "));
                break;
            } else {
                args.push(item.into())
            }
        }

        Self { prefix, command, args }
    }

    pub fn raw(&self) -> String {
        let mut args = Vec::with_capacity(self.args.len());
        for arg in &self.args {
            if !arg.contains(' ') {
                args.push(arg.into());
            } else {
                args.push(format!(":{}", arg));
                break;
            }
        }

        let args = args.join(" ");

        if let Some(x) = &self.prefix {
            format!(":{} {} {}\r\n", x.raw(), self.command, args)
        } else {
            format!("{} {}\r\n", self.command, args)
        }
    }

    pub fn from_message(message: CommonMessage) -> Self {
        match message {
            CommonMessage::Chat { channel, content, .. } => Self {
                prefix: None,
                command: "PRIVMSG".into(),
                args: vec![channel, content],
            },
            CommonMessage::Join { channel, .. } => Self {
                prefix: None,
                command: "JOIN".into(),
                args: vec![channel],
            },
        }
    }

    pub fn into_message(self) -> CommonMessage {
        match self.command.as_ref() {
            "PRIVMSG" => CommonMessage::Chat {
                channel: self.args[0].clone(),
                content: self.args[1].clone(),
                sender: self.prefix.unwrap().raw().into(),
            },
            "JOIN" => CommonMessage::Join {
                channel: self.args[0].clone(),
                sender: self.prefix.unwrap().raw().into(),
            },
            _ => panic!("Unhandled {}", self.command),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw().trim_matches(|x: char| x.is_control()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let message = Message::from_raw("NICK test\r\n".into());

        assert_eq!(message.command, "NICK");
        assert_eq!(message.args.len(), 1);
        assert_eq!(message.args[0], "test");
    }

    #[test]
    fn test_parse_trailing() {
        let message = Message::from_raw("PRIVMSG #test :test test\r\n".into());

        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.args.len(), 2);
        assert_eq!(message.args[0], "#test");
        assert_eq!(message.args[1], "test test");
    }

    #[test]
    fn test_parse_prefix() {
        let message = Message::from_raw(":test@test PRIVMSG #test :test test\r\n".into());

        assert!(message.prefix == Some(Prefix::User("test@test".into())));
        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.args.len(), 2);
        assert_eq!(message.args[0], "#test");
        assert_eq!(message.args[1], "test test");
    }

    #[test]
    fn test_parse_prefix_server() {
        let message = Message::from_raw(":server1.com PRIVMSG #test :test test\r\n".into());

        assert!(message.prefix == Some(Prefix::Server("server1.com".into())));
        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.args.len(), 2);
        assert_eq!(message.args[0], "#test");
        assert_eq!(message.args[1], "test test");
    }

    #[test]
    fn test_raw_simple() {
        let message = Message::new(None, "PING", vec!["12341234"]);

        assert_eq!(message.raw(), "PING 12341234\r\n");
    }

    #[test]
    fn test_raw_trailing() {
        let message = Message::new(Some(Prefix::from_raw("test@test".into())), "PRIVMSG", vec!["#test", "test test"]);

        assert_eq!(message.raw(), ":test@test PRIVMSG #test :test test\r\n");
    }

    #[test]
    fn test_raw_prefix() {
        let message = Message::new(Some(Prefix::from_raw("test@test".into())), "PING", vec!["12341234"]);

        assert_eq!(message.raw(), ":test@test PING 12341234\r\n");
    }
}
