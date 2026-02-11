use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStatus {
    Pending,
    Downloading(String), // Peer ID
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub index: u64,
    pub start: u64,
    pub end: u64,
    pub status: ChunkStatus,
    pub hash: Option<String>, // SHA-256
}

#[derive(Debug, Clone)]
pub struct FileTransfer {
    pub file_id: String,
    pub file_name: String,
    pub total_size: u64,
    pub chunks: Vec<Chunk>,
    pub output_path: String,
}

pub struct ChunkManager {
    transfers: Mutex<HashMap<String, FileTransfer>>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            transfers: Mutex::new(HashMap::new()),
        }
    }

    pub fn start_transfer(&self, file_name: String, total_size: u64, output_path: String) -> String {
        let file_id = Uuid::new_v4().to_string();
        let chunks = self.calculate_chunks(total_size);

        let transfer = FileTransfer {
            file_id: file_id.clone(),
            file_name,
            total_size,
            chunks,
            output_path,
        };

        let mut map = self.transfers.lock().unwrap();
        map.insert(file_id.clone(), transfer);
        
        println!("Started transfer {} with {} chunks", file_id, total_size / CHUNK_SIZE);
        file_id
    }

    fn calculate_chunks(&self, total_size: u64) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < total_size {
            let mut end = start + CHUNK_SIZE;
            if end > total_size {
                end = total_size;
            }

            chunks.push(Chunk {
                index,
                start,
                end,
                status: ChunkStatus::Pending,
                hash: None,
            });

            start = end;
            index += 1;
        }
        chunks
    }

    pub fn assign_chunk(&self, file_id: &str, peer_id: String) -> Option<Chunk> {
        let mut map = self.transfers.lock().unwrap();
        if let Some(transfer) = map.get_mut(file_id) {
            // Find first pending chunk
            if let Some(chunk) = transfer.chunks.iter_mut().find(|c| matches!(c.status, ChunkStatus::Pending)) {
                chunk.status = ChunkStatus::Downloading(peer_id);
                return Some(chunk.clone());
            }
        }
        None
    }
}
