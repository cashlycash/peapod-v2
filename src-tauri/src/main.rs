mod core;
mod chunk;

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;
use crate::core::{run_discovery, run_tcp_listener, AppState, Beacon, PeerEmitter};
use crate::chunk::ChunkManager;

// Event payload to Frontend
#[derive(Debug, Clone, serde::Serialize)]
struct PeerFound {
    id: String,
    name: String,
    ip: String, 
    port: u16,
}

struct TauriEmitter {
    handle: AppHandle,
}

impl PeerEmitter for TauriEmitter {
    fn emit(&self, peer: Beacon) {
         let event = PeerFound {
            id: peer.device_id,
            name: peer.name,
            ip: "unknown".to_string(), 
            port: peer.port,
        };
        let _ = self.handle.emit("peer-update", &event);
    }
}

#[tokio::main]
async fn main() {
    let chunk_manager = Arc::new(ChunkManager::new());
    let app_state = Arc::new(AppState {
        chunk_manager, // Init chunk manager
        ..Default::default()
    });
    
    let my_id = Uuid::new_v4().to_string();
    let my_name = "CashlyPod".to_string(); 

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_clone = app_state.clone();
            let id_clone = my_id.clone();
            let name_clone = my_name.clone();
            let tcp_id = my_id.clone();
            let tcp_state = app_state.clone();

            let emitter = TauriEmitter { handle };

            // Spawn Discovery Task
            tauri::async_runtime::spawn(async move {
                run_discovery(id_clone, name_clone, 45679, 45678, state_clone, emitter).await;
            });

            // Spawn TCP Listener
            tauri::async_runtime::spawn(async move {
                run_tcp_listener(tcp_id, 45679, tcp_state).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}