use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Chat { sender: String, channel: String, content: String },
    Join { sender: String, channel: String },
}
