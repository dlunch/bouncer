use std::iter;

pub struct Message {
    pub prefix: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

impl Message {
    pub fn new(prefix: Option<&str>, command: &str, args: Vec<&str>) -> Self {
        let prefix = prefix.map(|x| x.to_owned());
        let command = command.to_owned();
        let args = args.into_iter().map(|x| x.to_owned()).collect::<Vec<_>>();

        Self { prefix, command, args }
    }

    pub fn from_raw(raw: String) -> Self {
        let mut split = raw.split(' ').peekable();

        let prefix = if split.peek().unwrap().starts_with(':') {
            Some(split.next().unwrap()[1..].into())
        } else {
            None
        };

        let command = split.next().unwrap().into();

        let mut args = Vec::<String>::with_capacity(split.size_hint().0);
        while let Some(item) = split.next() {
            if item.starts_with(':') {
                args.push(iter::once(&item[1..]).chain(split).collect::<Vec<_>>().join(" "));
                break;
            } else {
                args.push(item.into())
            }
        }

        Self { prefix, command, args }
    }

    pub fn raw(&self) -> String {
        "".to_owned()
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let message = Message::from_raw("NICK test".into());

        assert_eq!(message.command, "NICK");
        assert_eq!(message.args.len(), 1);
        assert_eq!(message.args[0], "test");
    }

    #[test]
    fn test_parse_trailing() {
        let message = Message::from_raw("PRIVMSG #test :test test".into());

        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.args.len(), 2);
        assert_eq!(message.args[0], "#test");
        assert_eq!(message.args[1], "test test");
    }

    #[test]
    fn test_parse_prefix() {
        let message = Message::from_raw(":test@test PRIVMSG #test :test test".into());

        assert_eq!(message.prefix, Some("test@test".into()));
        assert_eq!(message.command, "PRIVMSG");
        assert_eq!(message.args.len(), 2);
        assert_eq!(message.args[0], "#test");
        assert_eq!(message.args[1], "test test");
    }
}
