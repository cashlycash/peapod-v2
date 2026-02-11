use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::protocol::Message;
use crate::state::AppState;

pub async fn run_tcp_listener(my_id: String, port: u16, state: Arc<AppState>) {
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
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_connection(socket, id_clone, true, state_clone).await;
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
            
            handle_connection(socket, my_id, false, state.clone()).await;
            
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

async fn send_message(socket: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let json = serde_json::to_string(msg).unwrap();
    let bytes = json.as_bytes();
    let len = bytes.len() as u32;
    socket.write_all(&len.to_le_bytes()).await?;
    socket.write_all(bytes).await?;
    Ok(())
}

async fn handle_connection(mut socket: TcpStream, my_id: String, _is_server: bool, state: Arc<AppState>) {
    // Send Handshake
    let handshake = Message::Handshake { version: 1, device_id: my_id.clone() };
    if let Err(e) = send_message(&mut socket, &handshake).await {
        eprintln!("Failed to send handshake: {}", e);
        return;
    }

    // Message Loop
    loop {
        let mut len_buf = [0u8; 4];
        if let Err(_) = socket.read_exact(&mut len_buf).await { return; }
        let msg_len = u32::from_le_bytes(len_buf) as usize;
        
        if msg_len > 10 * 1024 * 1024 { return; } // 10MB limit

        let mut buf = vec![0u8; msg_len];
        if let Err(_) = socket.read_exact(&mut buf).await { return; }

        if let Ok(msg) = serde_json::from_slice::<Message>(&buf) {
            match msg {
                Message::Handshake { version, device_id } => {
                    println!("[{}] Handshake received from {} (v{})", my_id, device_id, version);
                }
                Message::Ping => { let _ = send_message(&mut socket, &Message::Pong).await; }
                Message::Pong => {}
                Message::RequestChunk { file_id, index } => {
                    println!("[{}] Serving chunk {} for file {}", my_id, index, file_id);
                    // READ Chunk from Disk
                    if let Some(data) = state.chunk_manager.read_chunk(&file_id, index).await {
                        let response = Message::ChunkData { file_id, index, data };
                        let _ = send_message(&mut socket, &response).await;
                    } else {
                        eprintln!("Chunk not found!");
                    }
                }
                Message::ChunkData { file_id, index, data } => {
                    println!("[{}] Received chunk {} for file {} ({} bytes)", my_id, index, file_id, data.len());
                    // WRITE Chunk to Disk
                    state.chunk_manager.write_chunk(&file_id, index, data).await;
                }
            }
        }
    }
}
