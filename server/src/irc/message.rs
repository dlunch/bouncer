use std::iter;

// works as string enum
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod Reply {
    pub const RPL_NAMREPLY: &str = "353";
    pub const RPL_ENDOFNAMES: &str = "366";
    pub const RPL_ENDOFMOTD: &str = "376";
    pub const ERR_NOMOTD: &str = "422";
}

#[derive(Eq, PartialEq, Clone)]
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

#[derive(Clone)]
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
