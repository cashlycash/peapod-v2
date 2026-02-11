mod chunk;
mod discovery;
mod protocol;
mod state;
mod transport;

use clap::Parser;
use peapod::chunk::ChunkManager;
use peapod::discovery::run_discovery;
use peapod::protocol::Beacon;
use peapod::state::{AppState, PeerEmitter};
use peapod::transport::run_tcp_listener;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in headless daemon mode (no GUI)
    #[arg(long, short)]
    daemon: bool,
}

// GUI Emitter
struct TauriEmitter {
    handle: AppHandle,
}

impl PeerEmitter for TauriEmitter {
    fn emit(&self, peer: Beacon) {
        #[derive(Clone, serde::Serialize)]
        struct PeerFound {
            id: String,
            name: String,
            port: u16,
        }
        let event = PeerFound {
            id: peer.device_id,
            name: peer.name,
            port: peer.port,
        };
        let _ = self.handle.emit("peer-update", &event);
    }
}

// CLI Emitter
struct CliEmitter;
impl PeerEmitter for CliEmitter {
    fn emit(&self, peer: Beacon) {
        println!(">>> Discovered Peer: {} ({}) on port {}", peer.name, peer.device_id, peer.port);
    }
}

// Test Command (GUI only)
#[tauri::command]
async fn start_test_transfer(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let file_id = state.chunk_manager.start_transfer(
        "test.txt".into(),
        1024 * 1024 * 5,
        "/tmp/test_out.txt".into(),
    );
    Ok(format!("Started transfer: {}", file_id))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let chunk_manager = Arc::new(ChunkManager::new());
    let app_state = Arc::new(AppState {
        chunk_manager,
        ..Default::default()
    });

    let my_id = Uuid::new_v4().to_string();
    let my_name = if args.daemon { "CashlyPod-CLI".to_string() } else { "CashlyPod-GUI".to_string() };

    if args.daemon {
        println!("🚀 Starting PeaPod in HEADLESS DAEMON MODE");
        println!("ID: {}", my_id);
        
        let id_clone = my_id.clone();
        let name_clone = my_name.clone();
        let state_clone = app_state.clone();
        let tcp_state = app_state.clone();
        let tcp_id = my_id.clone();

        // Spawn TCP
        tokio::spawn(async move {
            run_tcp_listener(tcp_id, 45679, tcp_state).await;
        });

        // Run Discovery (Blocking main thread or await)
        run_discovery(
            id_clone, 
            name_clone, 
            45679, 
            45678, 
            state_clone, 
            CliEmitter
        ).await;

    } else {
        // GUI MODE
        tauri::Builder::default()
            .manage(app_state.clone())
            .invoke_handler(tauri::generate_handler![start_test_transfer])
            .setup(move |app| {
                let handle = app.handle().clone();
                let state_clone = app_state.clone();
                let id_clone = my_id.clone();
                let name_clone = my_name.clone();
                let tcp_id = my_id.clone();
                let tcp_state = app_state.clone();

                let emitter = TauriEmitter { handle };

                tauri::async_runtime::spawn(async move {
                    run_discovery(id_clone, name_clone, 45679, 45678, state_clone, emitter).await;
                });

                tauri::async_runtime::spawn(async move {
                    run_tcp_listener(tcp_id, 45679, tcp_state).await;
                });

                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
