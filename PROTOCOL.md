# PeaPod v2 Protocol Specification

## 1. Overview
PeaPod v2 uses a hybrid approach:
*   **UDP Multicast** for local peer discovery.
*   **TCP** for reliable control messages and data transfer.

## 2. Discovery (UDP)
*   **Address:** `239.255.60.60:45678`
*   **Format:** JSON
*   **Frequency:** Every 3-5 seconds.

### Payload
```json
{
  "device_id": "uuid-v4-string",
  "name": "Hostname",
  "port": 45679  // The TCP port for Phase 2 connections
}
```

## 3. Transport (TCP)
*   **Port:** 45679 (default, or dynamic as advertised in Beacon)
*   **Framing:** 4-byte Little Endian Length Header + Payload.

### Handshake (First Message)
When connecting, peers exchange identities.
```rust
struct Handshake {
    version: u8, // 2
    device_id: String,
}
```

### Message Types (Enum)
```rust
enum Message {
    Ping,
    Pong,
    RequestChunk { file_id: String, start: u64, end: u64 },
    ChunkData { file_id: String, start: u64, data: Vec<u8> },
}
```
*Serialization: Bincode (Rust) / Canonical JSON (Cross-platform MVP)*

## 4. Security (Planned)
*   TLS 1.3 or Noise Protocol Framework for TCP streams.
*   Discovery is unencrypted (public advertisement).
