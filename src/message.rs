use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    // Both directions
    Chat { sender: String, channel: String, content: String },
    // IRC to Client
    JoinedChannel { sender: String, channel: String },
    NamesList { channel: String, users: Vec<String> },
    NamesEnd { channel: String },
    // Client to IRC
    JoinChannel { channel: String },
}
