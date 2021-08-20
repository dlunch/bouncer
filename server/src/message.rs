use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    // Both directions
    Chat { sender: String, channel: String, content: String },
    // Source to Sink
    JoinedChannel { sender: String, channel: String },
    UsersList { channel: String, users: Vec<String> },
    // Sink to Source
    JoinChannel { channel: String },
}
