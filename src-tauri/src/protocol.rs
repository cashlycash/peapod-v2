use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Handshake { version: u8, device_id: String },
    Ping,
    Pong,
    RequestChunk { file_id: String, index: u64 },
    ChunkData { file_id: String, index: u64, data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub device_id: String,
    pub name: String,
    pub port: u16,
}
