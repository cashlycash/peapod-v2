# PeaPod v2 🫛🚀

> **"The next evolution of peer-to-peer swarming (now with 100% more Rust 🦀)"**

## What is this?

This is a complete rewrite of the PeaPod protocol. We're moving from the old architecture to a modern, modular stack:
*   **Core Logic:** Rust (Speed, Safety, 🦀)
*   **UI:** React + TypeScript (via Tauri v2)
*   **Vibe:** Immaculate

## The Grand Plan (Roadmap)

We are executing a 5-Phase Plan for World Domination™️ (or just file sharing):

1.  **Phase 1: Discovery UI (Current)**
    *   See nearby devices on the LAN.
    *   UDP Multicast/Broadcast beaconing.
    *   "Oh look, there's Harshit's laptop!"

2.  **Phase 2: Basic Communication**
    *   Establish reliable TCP/QUIC pipes between peers.
    *   Handshakes, Identity Exchange.

3.  **Phase 3: The Chunking**
    *   Split large files into tiny, edible peas (chunks).
    *   Manage state (Pending, Downloading, Done).

4.  **Phase 4: Distribution (The Swarm)**
    *   Assign chunks to peers.
    *   "Hey you, download bytes 0-1MB for me."

5.  **Phase 5: Assembly**
    *   Put Humpty Dumpty back together again.
    *   Verify hashes, write to disk.

## How to Run

### Prerequisites
*   **Node.js** (v18+)
*   **Rust** (stable)
*   **Tauri CLI** (`cargo install tauri-cli --version "^2.0.0-beta"`)

### Dev Mode
```bash
npm install
npm run tauri dev
```

### Build
```bash
npm run tauri build
```

---
*Maintained by CashlyBot (and hopefully Harshit if he reads this).*
