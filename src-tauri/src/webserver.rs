use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use peapod::state::AppState;
use uuid::Uuid;

#[derive(Serialize)]
struct PeerInfo {
    id: String,
    name: String,
    port: u16,
    is_connected: bool,
}

#[derive(Serialize)]
struct StatusResponse {
    peers: Vec<PeerInfo>,
    active_transfers: usize,
}

#[derive(Deserialize)]
struct TransferRequest {
    file_path: String,
    file_size: u64,
    output_path: String,
}

#[derive(Serialize)]
struct TransferResponse {
    transfer_id: String,
    status: String,
}

pub async fn start_webserver(state: Arc<AppState>) {
    // Create the web server router
    let app = Router::new()
        .route("/status", get(get_status))
        .route("/transfer", post(start_transfer))
        .with_state(state);

    // Bind to all interfaces (0.0.0.0) on port 8080
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    println!("🚀 Webserver started on http://0.0.0.0:8080");
    println!("   - Status endpoint: GET /status");
    println!("   - Transfer endpoint: POST /transfer");

    // Run the server
    axum::serve(listener, app).await.unwrap();
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let peers = state.peers.lock().unwrap();
    let active_connections = state.active_connections.lock().unwrap();

    let peer_info: Vec<PeerInfo> = peers
        .values()
        .map(|beacon| PeerInfo {
            id: beacon.device_id.clone(),
            name: beacon.name.clone(),
            port: beacon.port,
            is_connected: active_connections.get(&beacon.device_id).copied().unwrap_or(false),
        })
        .collect();

    let active_transfers = state.chunk_manager.get_active_transfers();

    let response = StatusResponse {
        peers: peer_info,
        active_transfers,
    };

    Json(response)
}

async fn start_transfer(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<TransferRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // In a real implementation, this would start a file transfer
    // For now, we'll just simulate it
    let transfer_id = uuid::Uuid::new_v4().to_string();

    // This is where we would actually start the transfer using the chunk manager
    // For now, we just return a success response
    let response = TransferResponse {
        transfer_id,
        status: "started".to_string(),
    };

    Ok(Json(response))
}