use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Chat { sender: String, channel: String, content: String },
    JoinedChannel { sender: String, channel: String },
    JoinChannel { channel: String },
}
