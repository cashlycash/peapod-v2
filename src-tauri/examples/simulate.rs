use peapod::discovery::run_discovery;
use peapod::protocol::Beacon;
use peapod::state::{AppState, PeerEmitter};
use peapod::transport::run_tcp_listener;
use peapod::chunk::ChunkManager;
use std::sync::Arc;
use tokio::time::Duration;

struct ConsoleEmitter {
    owner: String,
}

impl PeerEmitter for ConsoleEmitter {
    fn emit(&self, peer: Beacon) {
        println!(
            "[{}] UI Event: Discovered peer {} on port {}",
            self.owner, peer.name, peer.port
        );
    }
}

#[tokio::main]
async fn main() {
    println!("Starting PeaPod Simulation...");

    // NODE A
    let id_a = "NODE-A".to_string();
    let port_a = 50001;
    let chunk_manager_a = Arc::new(ChunkManager::new());
    let state_a = Arc::new(AppState {
        chunk_manager: chunk_manager_a,
        ..Default::default()
    });
    let emitter_a = ConsoleEmitter { owner: "A".into() };

    // NODE B
    let id_b = "NODE-B".to_string();
    let port_b = 50002;
    let chunk_manager_b = Arc::new(ChunkManager::new());
    let state_b = Arc::new(AppState {
        chunk_manager: chunk_manager_b,
        ..Default::default()
    });
    let emitter_b = ConsoleEmitter { owner: "B".into() };

    // Start TCP Listeners
    let id_a_clone = id_a.clone();
    let state_a_tcp = state_a.clone();
    tokio::spawn(async move {
        run_tcp_listener(id_a_clone, port_a, state_a_tcp).await;
    });

    let id_b_clone = id_b.clone();
    let state_b_tcp = state_b.clone();
    tokio::spawn(async move {
        run_tcp_listener(id_b_clone, port_b, state_b_tcp).await;
    });

    // Allow listeners to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Start Discovery (Both on UDP 45678)
    let id_a_clone = id_a.clone();
    let state_a_clone = state_a.clone();
    tokio::spawn(async move {
        run_discovery(
            id_a_clone,
            "Node A".into(),
            port_a,
            45678,
            state_a_clone,
            emitter_a,
        )
        .await;
    });

    let id_b_clone = id_b.clone();
    let state_b_clone = state_b.clone();
    tokio::spawn(async move {
        run_discovery(
            id_b_clone,
            "Node B".into(),
            port_b,
            45678,
            state_b_clone,
            emitter_b,
        )
        .await;
    });

    // Run for 15 seconds then exit
    tokio::time::sleep(Duration::from_secs(15)).await;
    println!("Simulation finished.");
}
