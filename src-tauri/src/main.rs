use tauri::Manager;
use tokio::net::UdpSocket;
use std::time::Duration;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Beacon: The "Hello" message
#[derive(Debug, Serialize, Deserialize)]
struct Beacon {
    device_id: String,
    name: String,
    port: u16,
}

#[derive(Default)]
struct AppState {
    peers: Mutex<HashMap<String, Beacon>>, // Map device_id -> Beacon
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState::default());
    let my_id = Uuid::new_v4().to_string();
    let my_name = "CashlyPod".to_string(); // Temporary static name

    // Spawn Discovery Task
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        run_discovery(my_id, my_name, state_clone).await;
    });

    tauri::Builder::default()
        .setup(|app| {
            // Can access app handle here to emit to frontend
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn run_discovery(my_id: String, my_name: String, state: Arc<AppState>) {
    let socket = UdpSocket::bind("0.0.0.0:45678").await.unwrap();
    socket.set_broadcast(true).unwrap();
    // Multicast group joining logic would go here (platform-specific often)
    // For now, simpler UDP broadcast to 255.255.255.255:45678

    let beacon = Beacon {
        device_id: my_id.clone(),
        name: my_name.clone(),
        port: 45679, // TCP port (TBD)
    };
    let beacon_json = serde_json::to_string(&beacon).unwrap();
    
    // Broadcast Loop (Sender)
    let socket_send = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    socket_send.set_broadcast(true).unwrap();
    let msg = beacon_json.clone();
    
    tokio::spawn(async move {
        loop {
            socket_send.send_to(msg.as_bytes(), "255.255.255.255:45678").await.unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Listen Loop (Receiver)
    let mut buf = [0; 1024];
    loop {
        if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
            if let Ok(peer_beacon) = serde_json::from_slice::<Beacon>(&buf[..len]) {
                if peer_beacon.device_id != my_id {
                    let mut peers = state.peers.lock().unwrap();
                    if !peers.contains_key(&peer_beacon.device_id) {
                        println!("New Peer Discovered: {} ({:?})", peer_beacon.name, addr);
                        // TODO: Emit to frontend
                    }
                    peers.insert(peer_beacon.device_id.clone(), peer_beacon);
                }
            }
        }
    }
}