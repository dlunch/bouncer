use std::fmt;

pub struct Message {
    raw: String,
}

impl Message {
    pub fn new(raw: String) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &[u8] {
        self.raw.as_bytes()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}
