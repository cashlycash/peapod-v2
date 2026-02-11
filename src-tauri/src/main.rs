use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use tauri::{AppHandle, Emitter, Manager};
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
            let tcp_id = my_id.clone();

            // Spawn Discovery Task
            tauri::async_runtime::spawn(async move {
                run_discovery(id_clone, name_clone, state_clone, handle).await;
            });

            // Spawn TCP Listener
            tauri::async_runtime::spawn(async move {
                run_tcp_listener(tcp_id, TCP_PORT).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn run_tcp_listener(my_id: String, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    // Bind TCP Listener
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind TCP listener on {}: {}", addr, e);
            return;
        }
    };
    println!("TCP Listener running on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("New TCP connection from: {:?}", addr);
                let id_clone = my_id.clone();
                tokio::spawn(async move {
                    handle_connection(socket, id_clone).await;
                });
            }
            Err(e) => eprintln!("TCP Accept error: {}", e),
        }
    }
}

async fn handle_connection(mut socket: TcpStream, my_id: String) {
    // Basic Handshake: Send ID length (u32) + ID bytes
    let id_bytes = my_id.as_bytes();
    let len = id_bytes.len() as u32;
    
    if let Err(e) = socket.write_all(&len.to_le_bytes()).await {
        eprintln!("Failed to send handshake len: {}", e);
        return;
    }
    if let Err(e) = socket.write_all(id_bytes).await {
        eprintln!("Failed to send handshake ID: {}", e);
        return;
    }

    // Read Peer Handshake
    let mut len_buf = [0u8; 4];
    if let Err(_) = socket.read_exact(&mut len_buf).await {
        return;
    }
    let peer_len = u32::from_le_bytes(len_buf) as usize;
    
    // Safety check on length (max 1KB for ID)
    if peer_len > 1024 {
        eprintln!("Peer ID too long: {}", peer_len);
        return;
    }

    let mut peer_id_buf = vec![0u8; peer_len];
    if let Err(_) = socket.read_exact(&mut peer_id_buf).await {
        return;
    }
    let peer_id = String::from_utf8_lossy(&peer_id_buf);
    
    println!("Handshake success with peer: {}", peer_id);
    
    // Keep connection open (Echo loop for now)
    let mut buf = [0u8; 1024];
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(n) if n == 0 => return, // Closed
            Ok(n) => n,
            Err(_) => return,
        };
        
        // Echo back
        if let Err(_) = socket.write_all(&buf[0..n]).await {
            return;
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
    let listener = match create_multicast_socket() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create multicast socket: {}", e);
            return;
        }
    };
    let listener = UdpSocket::from_std(listener.into()).expect("Failed to convert to Tokio socket");

    // Setup Sender Socket (Broadcast)
    let sender = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind sender");
    sender.set_broadcast(true).expect("Failed to set broadcast");

    let beacon = Beacon {
        device_id: my_id.clone(),
        name: my_name.clone(),
        port: TCP_PORT,
    };
    let beacon_json = serde_json::to_string(&beacon).unwrap();
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
                if let Ok(peer_beacon) = serde_json::from_slice::<Beacon>(&buf[..len]) {
                    if peer_beacon.device_id != my_id {
                        let mut peers = state.peers.lock().unwrap();
                        if !peers.contains_key(&peer_beacon.device_id) {
                            println!("New Peer Discovered: {} ({:?})", peer_beacon.name, addr);
                            
                            // Emit to Frontend
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
    socket.set_reuse_address(true)?;
    #[cfg(not(target_os = "windows"))]
    socket.set_reuse_port(true)?;

    let addr: SocketAddr = format!("0.0.0.0:{}", PORT).parse().unwrap();
    socket.bind(&addr.into())?;

    let multi_addr: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    let interface = Ipv4Addr::new(0, 0, 0, 0); 
    socket.join_multicast_v4(&multi_addr, &interface)?;

    socket.set_nonblocking(true)?;
    Ok(socket)
}