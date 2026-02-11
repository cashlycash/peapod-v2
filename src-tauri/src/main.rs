use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use tauri::{AppHandle, Manager, Emitter};
use tokio::net::UdpSocket;
use uuid::Uuid;

const MULTICAST_ADDR: &str = "239.255.60.60";
const PORT: u16 = 45678;

// Beacon: The "Hello" message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Beacon {
    device_id: String,
    name: String,
    port: u16,
}

// Event payload to Frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerFound {
    id: String,
    name: String,
    ip: String,
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
    let my_name = "CashlyPod".to_string(); // TODO: Get hostname

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_clone = app_state.clone();
            let id_clone = my_id.clone();
            let name_clone = my_name.clone();

            // Spawn Discovery Task
            tauri::async_runtime::spawn(async move {
                run_discovery(id_clone, name_clone, state_clone, handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn run_discovery(
    my_id: String,
    my_name: String,
    state: Arc<AppState>,
    app_handle: AppHandle,
) {
    println!("Starting discovery on {}:{}", MULTICAST_ADDR, PORT);

    // Setup Listener Socket (Receive)
    let listener = create_multicast_socket().expect("Failed to create multicast listener");
    let listener = UdpSocket::from_std(listener.into()).expect("Failed to convert to Tokio socket");

    // Setup Sender Socket (Broadcast)
    let sender = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind sender");
    sender.set_broadcast(true).expect("Failed to set broadcast");
    // Multicast loopback enabled by default usually, good for testing on one machine.

    let beacon = Beacon {
        device_id: my_id.clone(),
        name: my_name.clone(),
        port: 45679, // TCP port (Placeholder for Phase 2)
    };
    let beacon_json = serde_json::to_string(&beacon).unwrap();
    let msg_bytes = beacon_json.as_bytes();
    let target_addr: SocketAddr = format!("{}:{}", MULTICAST_ADDR, PORT).parse().unwrap();

    // Spawn Sender Loop
    let sender_msg = beacon_json.clone();
    let sender_socket = Arc::new(sender);
    let sender_clone = sender_socket.clone();
    
    tokio::spawn(async move {
        loop {
            if let Err(e) = sender_clone.send_to(sender_msg.as_bytes(), target_addr).await {
                eprintln!("Failed to send beacon: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // Listen Loop
    let mut buf = [0; 1024];
    loop {
        match listener.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                // Ignore our own beacons? (Optional)
                // If we want to test on one machine, we keep them.
                
                if let Ok(peer_beacon) = serde_json::from_slice::<Beacon>(&buf[..len]) {
                    // Log discovery
                    if peer_beacon.device_id != my_id {
                        let mut peers = state.peers.lock().unwrap();
                        if !peers.contains_key(&peer_beacon.device_id) {
                            println!("New Peer Discovered: {} ({:?})", peer_beacon.name, addr);
                            
                            // Emit to Frontend
                            // Event: "peer-update"
                            let event = PeerFound {
                                id: peer_beacon.device_id.clone(),
                                name: peer_beacon.name.clone(),
                                ip: addr.ip().to_string(),
                                port: peer_beacon.port,
                            };
                            
                            if let Err(e) = app_handle.emit("peer-update", &event) {
                                eprintln!("Failed to emit event: {}", e);
                            }
                        }
                        // Update last seen timestamp (TODO)
                        peers.insert(peer_beacon.device_id.clone(), peer_beacon);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving UDP packet: {}", e);
            }
        }
    }
}

fn create_multicast_socket() -> std::io::Result<Socket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    
    // Reuse address/port is crucial for multiple apps on same machine binding to multicast port
    socket.set_reuse_address(true)?;
    #[cfg(not(target_os = "windows"))]
    socket.set_reuse_port(true)?; // Linux/macOS specific

    // Bind to 0.0.0.0:PORT
    let addr: SocketAddr = format!("0.0.0.0:{}", PORT).parse().unwrap();
    socket.bind(&addr.into())?;

    // Join Multicast Group
    let multi_addr: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    let interface = Ipv4Addr::new(0, 0, 0, 0); // ANY interface
    socket.join_multicast_v4(&multi_addr, &interface)?;

    socket.set_nonblocking(true)?;

    Ok(socket)
}