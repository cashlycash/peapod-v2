use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::Beacon;
use crate::chunk::ChunkManager;

pub trait PeerEmitter: Send + Sync + 'static {
    fn emit(&self, peer: Beacon);
}

#[derive(Default)]
pub struct AppState {
    pub peers: Mutex<HashMap<String, Beacon>>, 
    pub active_connections: Mutex<HashMap<String, bool>>,
    pub chunk_manager: Arc<ChunkManager>,
}
