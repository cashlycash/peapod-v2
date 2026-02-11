use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

pub const MULTICAST_ADDR: &str = "239.255.60.60";

// Beacon: The "Hello" message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub device_id: String,
    pub name: String,
    pub port: u16,
}

// Abstract Event Emitter trait to decouple from Tauri
pub trait PeerEmitter: Send + Sync + 'static {
    fn emit(&self, peer: Beacon);
}

#[derive(Default)]
pub struct AppState {
    pub peers: Mutex<HashMap<String, Beacon>>, 
    pub active_connections: Mutex<HashMap<String, bool>>,
}

pub async fn run_tcp_listener(my_id: String, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind TCP listener on {}: {}", addr, e);
            return;
        }
    };
    println!("[{}] TCP Listener running on {}", my_id, addr);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[{}] New incoming TCP connection from: {:?}", my_id, addr);
                let id_clone = my_id.clone();
                tokio::spawn(async move {
                    handle_connection(socket, id_clone, true).await;
                });
            }
            Err(e) => eprintln!("TCP Accept error: {}", e),
        }
    }
}

pub async fn connect_to_peer(peer_id: String, ip: String, port: u16, my_id: String, state: Arc<AppState>) {
    let addr = format!("{}:{}", ip, port);
    println!("[{}] Attempting to connect to peer {} at {}", my_id, peer_id, addr);

    match TcpStream::connect(&addr).await {
        Ok(socket) => {
            println!("[{}] Connected to peer {}", my_id, peer_id);
            {
                let mut conns = state.active_connections.lock().unwrap();
                conns.insert(peer_id.clone(), true);
            }
            
            handle_connection(socket, my_id, false).await;
            
            {
                let mut conns = state.active_connections.lock().unwrap();
                conns.remove(&peer_id);
            }
            println!("Disconnected from peer {}", peer_id);
        }
        Err(e) => {
            eprintln!("[{}] Failed to connect to peer {}: {}", my_id, peer_id, e);
        }
    }
}

async fn handle_connection(mut socket: TcpStream, my_id: String, is_server: bool) {
    let id_bytes = my_id.as_bytes();
    let len = id_bytes.len() as u32;
    
    if let Err(_) = socket.write_all(&len.to_le_bytes()).await { return; }
    if let Err(_) = socket.write_all(id_bytes).await { return; }

    let mut len_buf = [0u8; 4];
    if let Err(_) = socket.read_exact(&mut len_buf).await { return; }
    let peer_len = u32::from_le_bytes(len_buf) as usize;
    
    if peer_len > 1024 { return; }

    let mut peer_id_buf = vec![0u8; peer_len];
    if let Err(_) = socket.read_exact(&mut peer_id_buf).await { return; }
    let peer_id = String::from_utf8_lossy(&peer_id_buf);
    
    println!("[{}] Handshake success. Connected to: {} (Server: {})", my_id, peer_id, is_server);
    
    // Keep alive
    let mut buf = [0u8; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => return,
            Ok(n) => {
                if let Err(_) = socket.write_all(&buf[0..n]).await { return; }
            }
            Err(_) => return,
        }
    }
}

pub async fn run_discovery<E: PeerEmitter>(
    my_id: String,
    my_name: String,
    my_port: u16,
    discovery_port: u16,
    state: Arc<AppState>,
    emitter: E,
) {
    println!("[{}] Starting discovery on multicast:{}", my_id, discovery_port);

    let listener = match create_multicast_socket(discovery_port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create multicast socket: {}", e);
            return;
        }
    };
    let listener = UdpSocket::from_std(listener.into()).expect("Failed to convert to Tokio socket");

    let sender = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind sender");
    sender.set_broadcast(true).expect("Failed to set broadcast");

    let beacon = Beacon {
        device_id: my_id.clone(),
        name: my_name.clone(),
        port: my_port,
    };
    let beacon_json = serde_json::to_string(&beacon).unwrap();
    let target_addr: SocketAddr = format!("{}:{}", MULTICAST_ADDR, discovery_port).parse().unwrap();

    let sender_msg = beacon_json.clone();
    let sender_clone = Arc::new(sender);
    
    // Sender Task
    tokio::spawn(async move {
        loop {
            let _ = sender_clone.send_to(sender_msg.as_bytes(), target_addr).await;
            tokio::time::sleep(Duration::from_secs(2)).await; // Fast beacon for test
        }
    });

    // Listener Loop
    let mut buf = [0; 1024];
    loop {
        if let Ok((len, addr)) = listener.recv_from(&mut buf).await {
            if let Ok(peer_beacon) = serde_json::from_slice::<Beacon>(&buf[..len]) {
                if peer_beacon.device_id != my_id {
                    let mut peers = state.peers.lock().unwrap();
                    let mut should_connect = false;

                    if !peers.contains_key(&peer_beacon.device_id) {
                        println!("[{}] Discovered: {} ({:?})", my_id, peer_beacon.name, addr);
                        should_connect = true;
                        emitter.emit(peer_beacon.clone());
                    }
                    peers.insert(peer_beacon.device_id.clone(), peer_beacon.clone());
                    
                    if should_connect {
                        let mut conns = state.active_connections.lock().unwrap();
                        if !conns.contains_key(&peer_beacon.device_id) {
                            conns.insert(peer_beacon.device_id.clone(), true);
                            let peer_id = peer_beacon.device_id.clone();
                            // Use the detected IP (addr.ip()) but the advertised PORT
                            let peer_ip = addr.ip().to_string(); 
                            let peer_port = peer_beacon.port;
                            let my_id_clone = my_id.clone();
                            let state_clone = state.clone();

                            tokio::spawn(async move {
                                connect_to_peer(peer_id, peer_ip, peer_port, my_id_clone, state_clone).await;
                            });
                        }
                    }
                }
            }
        }
    }
}

fn create_multicast_socket(port: u16) -> std::io::Result<Socket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(not(target_os = "windows"))]
    socket.set_reuse_port(true)?;

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    socket.bind(&addr.into())?;

    let multi_addr: Ipv4Addr = MULTICAST_ADDR.parse().unwrap();
    let interface = Ipv4Addr::new(0, 0, 0, 0); 
    socket.join_multicast_v4(&multi_addr, &interface)?;

    socket.set_nonblocking(true)?;
    Ok(socket)
}