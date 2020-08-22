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

    pub fn from_raw(_: String) -> Self {
        Self {
            prefix: None,
            command: "".to_owned(),
            args: vec![],
        }
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
