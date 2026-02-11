use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use tauri::{AppHandle, Manager, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use uuid::Uuid;

const MULTICAST_ADDR: &str = "239.255.60.60";
const PORT: u16 = 45678;
const TCP_PORT: u16 = 45679;

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

// Phase 2: Handshake message exchanged over TCP
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Handshake {
    device_id: String,
    name: String,
    version: String,
}

// Phase 2: Connection status event sent to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionEvent {
    peer_id: String,
    status: String, // "connected" | "disconnected" | "error"
}

struct AppState {
    peers: Mutex<HashMap<String, Beacon>>,
    connected_peers: Mutex<HashMap<String, String>>, // peer_id -> ip:port
    my_id: String,
    my_name: String,
}

#[tokio::main]
async fn main() {
    let my_id = Uuid::new_v4().to_string();
    let my_name = "CashlyPod".to_string(); // TODO: Get hostname

    let app_state = Arc::new(AppState {
        peers: Mutex::new(HashMap::new()),
        connected_peers: Mutex::new(HashMap::new()),
        my_id: my_id.clone(),
        my_name: my_name.clone(),
    });

    let state_for_tauri = app_state.clone();

    tauri::Builder::default()
        .manage(state_for_tauri)
        .invoke_handler(tauri::generate_handler![connect_to_peer, disconnect_from_peer, get_connections])
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_clone = app_state.clone();
            let id_clone = my_id.clone();
            let name_clone = my_name.clone();

            // Spawn Discovery Task
            let discovery_handle = handle.clone();
            let discovery_state = state_clone.clone();
            let discovery_id = id_clone.clone();
            let discovery_name = name_clone.clone();
            tauri::async_runtime::spawn(async move {
                run_discovery(discovery_id, discovery_name, discovery_state, discovery_handle).await;
            });

            // Spawn TCP Listener (Phase 2)
            let tcp_handle = handle.clone();
            let tcp_state = state_clone.clone();
            let tcp_id = id_clone.clone();
            let tcp_name = name_clone.clone();
            tauri::async_runtime::spawn(async move {
                run_tcp_listener(tcp_id, tcp_name, tcp_state, tcp_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Phase 2: Tauri command - connect to a discovered peer
#[tauri::command]
async fn connect_to_peer(
    peer_id: String,
    ip: String,
    port: u16,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // Check if already connected
    {
        let connected = state.connected_peers.lock().unwrap();
        if connected.contains_key(&peer_id) {
            return Err("Already connected to this peer".into());
        }
    }

    let addr = format!("{}:{}", ip, port);
    let my_id = state.my_id.clone();
    let my_name = state.my_name.clone();

    // Attempt TCP connection
    let stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Perform handshake
    let handshake = Handshake {
        device_id: my_id,
        name: my_name,
        version: "0.1.0".to_string(),
    };

    match perform_handshake(stream, &handshake).await {
        Ok(remote_handshake) => {
            // Store connection
            {
                let mut connected = state.connected_peers.lock().unwrap();
                connected.insert(peer_id.clone(), addr);
            }

            // Emit connection event to frontend
            let event = ConnectionEvent {
                peer_id: peer_id.clone(),
                status: "connected".to_string(),
            };
            let _ = app_handle.emit("connection-status", &event);

            Ok(format!("Connected to {}", remote_handshake.name))
        }
        Err(e) => {
            let event = ConnectionEvent {
                peer_id: peer_id.clone(),
                status: "error".to_string(),
            };
            let _ = app_handle.emit("connection-status", &event);
            Err(format!("Handshake failed: {}", e))
        }
    }
}

// Phase 2: Tauri command - disconnect from a peer
#[tauri::command]
async fn disconnect_from_peer(
    peer_id: String,
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
) -> Result<String, String> {
    {
        let mut connected = state.connected_peers.lock().unwrap();
        if connected.remove(&peer_id).is_none() {
            return Err("Not connected to this peer".into());
        }
    }

    let event = ConnectionEvent {
        peer_id,
        status: "disconnected".to_string(),
    };
    let _ = app_handle.emit("connection-status", &event);

    Ok("Disconnected".to_string())
}

// Phase 2: Tauri command - get current connections
#[tauri::command]
fn get_connections(state: State<'_, Arc<AppState>>) -> Vec<String> {
    let connected = state.connected_peers.lock().unwrap();
    connected.keys().cloned().collect()
}

// Phase 2: Perform TCP handshake (send our identity, receive theirs)
async fn perform_handshake(
    mut stream: TcpStream,
    local: &Handshake,
) -> Result<Handshake, String> {
    let payload = serde_json::to_vec(local).map_err(|e| e.to_string())?;
    let len = (payload.len() as u32).to_be_bytes();

    // Send length-prefixed handshake
    stream
        .write_all(&len)
        .await
        .map_err(|e| format!("Write length failed: {}", e))?;
    stream
        .write_all(&payload)
        .await
        .map_err(|e| format!("Write payload failed: {}", e))?;

    // Read remote handshake
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("Read length failed: {}", e))?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    if msg_len > 4096 {
        return Err("Handshake message too large".into());
    }

    let mut msg_buf = vec![0u8; msg_len];
    stream
        .read_exact(&mut msg_buf)
        .await
        .map_err(|e| format!("Read payload failed: {}", e))?;

    let remote: Handshake =
        serde_json::from_slice(&msg_buf).map_err(|e| format!("Parse handshake failed: {}", e))?;

    Ok(remote)
}

// Phase 2: TCP listener - accepts incoming peer connections
async fn run_tcp_listener(
    my_id: String,
    my_name: String,
    state: Arc<AppState>,
    app_handle: AppHandle,
) {
    let addr = format!("0.0.0.0:{}", TCP_PORT);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    println!("TCP listener started on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                println!("Incoming TCP connection from {}", peer_addr);

                let id = my_id.clone();
                let name = my_name.clone();
                let state = state.clone();
                let handle = app_handle.clone();

                tokio::spawn(async move {
                    handle_incoming_connection(stream, peer_addr, id, name, state, handle).await;
                });
            }
            Err(e) => {
                eprintln!("TCP accept error: {}", e);
            }
        }
    }
}

async fn handle_incoming_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    my_id: String,
    my_name: String,
    state: Arc<AppState>,
    app_handle: AppHandle,
) {
    let local_handshake = Handshake {
        device_id: my_id,
        name: my_name,
        version: "0.1.0".to_string(),
    };

    match perform_handshake(stream, &local_handshake).await {
        Ok(remote) => {
            println!(
                "Handshake complete with {} ({})",
                remote.name, remote.device_id
            );

            // Store connection
            {
                let mut connected = state.connected_peers.lock().unwrap();
                connected.insert(remote.device_id.clone(), peer_addr.to_string());
            }

            let event = ConnectionEvent {
                peer_id: remote.device_id,
                status: "connected".to_string(),
            };
            let _ = app_handle.emit("connection-status", &event);
        }
        Err(e) => {
            eprintln!("Incoming handshake failed from {}: {}", peer_addr, e);
        }
    }
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
        port: TCP_PORT,
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