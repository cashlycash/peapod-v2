use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;
use crate::protocol::Beacon;
use crate::state::{AppState, PeerEmitter};
use crate::transport::connect_to_peer;

pub const MULTICAST_ADDR: &str = "239.255.60.60";

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
        Err(_) => return,
    };
    let listener = UdpSocket::from_std(listener.into()).unwrap();
    let sender = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    sender.set_broadcast(true).unwrap();

    let beacon = Beacon { device_id: my_id.clone(), name: my_name.clone(), port: my_port };
    let beacon_json = serde_json::to_string(&beacon).unwrap();
    let target_addr: SocketAddr = format!("{}:{}", MULTICAST_ADDR, discovery_port).parse().unwrap();

    let sender_msg = beacon_json.clone();
    let sender_clone = Arc::new(sender);
    tokio::spawn(async move {
        loop {
            let _ = sender_clone.send_to(sender_msg.as_bytes(), target_addr).await;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

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
