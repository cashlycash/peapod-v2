use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use uuid::Uuid;

pub const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkStatus {
    Pending,
    Downloading(String),
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub index: u64,
    pub start: u64,
    pub end: u64,
    pub status: ChunkStatus,
    pub hash: Option<String>, 
}

#[derive(Debug, Clone)]
pub struct FileTransfer {
    pub file_id: String,
    pub file_name: String,
    pub total_size: u64,
    pub chunks: Vec<Chunk>,
    pub output_path: String,
    pub source_path: Option<String>, 
}

#[derive(Default)]
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
            source_path: None,
        };

        let mut map = self.transfers.lock().unwrap();
        map.insert(file_id.clone(), transfer);
        file_id
    }
    
    pub fn register_source_file(&self, path: String, total_size: u64) -> String {
        let file_id = Uuid::new_v4().to_string();
        let chunks = self.calculate_chunks(total_size);
        let transfer = FileTransfer {
            file_id: file_id.clone(),
            file_name: "source".into(),
            total_size,
            chunks,
            output_path: "".into(),
            source_path: Some(path),
        };
        let mut map = self.transfers.lock().unwrap();
        map.insert(file_id.clone(), transfer);
        file_id
    }

    fn calculate_chunks(&self, total_size: u64) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;
        while start < total_size {
            let mut end = start + CHUNK_SIZE;
            if end > total_size { end = total_size; }
            chunks.push(Chunk { index, start, end, status: ChunkStatus::Pending, hash: None });
            start = end;
            index += 1;
        }
        chunks
    }

    // READ (Serve)
    pub async fn read_chunk(&self, file_id: &str, index: u64) -> Option<Vec<u8>> {
        let (path, start, len) = {
            let map = self.transfers.lock().unwrap();
            let t = map.get(file_id)?;
            let chunk = t.chunks.get(index as usize)?;
            let path = t.source_path.clone().or_else(|| Some(t.output_path.clone()))?;
            (path, chunk.start, chunk.end - chunk.start)
        };

        if let Ok(mut file) = File::open(path).await {
            if let Ok(_) = file.seek(SeekFrom::Start(start)).await {
                let mut buf = vec![0u8; len as usize];
                if let Ok(_) = file.read_exact(&mut buf).await {
                    return Some(buf);
                }
            }
        }
        None
    }

    // WRITE (Receive)
    pub async fn write_chunk(&self, file_id: &str, index: u64, data: Vec<u8>) {
        let (path, start) = {
            let map = self.transfers.lock().unwrap();
            if let Some(t) = map.get(file_id) {
                if let Some(c) = t.chunks.get(index as usize) {
                    (t.output_path.clone(), c.start)
                } else { return; }
            } else { return; }
        };

        if let Ok(mut file) = fs::OpenOptions::new().write(true).create(true).open(path).await {
            if let Ok(_) = file.seek(SeekFrom::Start(start)).await {
                let _ = file.write_all(&data).await;
                let mut map = self.transfers.lock().unwrap();
                if let Some(t) = map.get_mut(file_id) {
                    if let Some(c) = t.chunks.get_mut(index as usize) {
                        c.status = ChunkStatus::Completed;
                    }
                }
            }
        }
    }
}
