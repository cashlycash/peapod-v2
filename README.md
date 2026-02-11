# PeaPod v2 🫛🚀

> **⚠️ STATUS: ALPHA (v0.6.0+)**
> Functional P2P Swarm. Discovery, Transport, and Chunking are live.
> Expect bugs, but it works.

![Test Suite](https://github.com/cashlycash/peapod-v2/actions/workflows/test.yml/badge.svg)

**PeaPod** is a local peer-to-peer swarming protocol. It pools internet connections from nearby devices to speed up downloads.

## 📥 Installation

**Latest Release:** [Check Releases Page](https://github.com/cashlycash/peapod-v2/releases)

### One-Line Installers
*Run this in your terminal:*

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/cashlycash/peapod-v2/master/install.sh | sh
```

**Windows (PowerShell):**
```powershell
iwr https://raw.githubusercontent.com/cashlycash/peapod-v2/master/install.ps1 | iex
```

## 🛠️ Build from Source

### Prerequisites
*   **Node.js** (v20+)
*   **Rust** (stable)
*   **Tauri CLI** (`cargo install tauri-cli`)
*   **Linux deps:** `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `libssl-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`

### Steps
1.  Clone: `git clone https://github.com/cashlycash/peapod-v2.git`
2.  Setup: `npm install`
3.  Run: `npm run tauri dev`

## 🗺️ Feature Status

*   ✅ **Phase 1: Discovery** (UDP Multicast Beaconing)
*   ✅ **Phase 2: Transport** (TCP, Handshake, Connection Management)
*   ✅ **Phase 3: Chunking** (File Splitting, State Tracking)
*   ✅ **Phase 4: Distribution** (Requesting & Serving Chunks)
*   ✅ **Phase 5: Assembly** (Writing Chunks to Disk)
*   ✅ **UI:** Industrial Dashboard + Real-time Peer List

## Architecture

- **Core:** Rust (Tokio/Tauri)
- **Frontend:** React + TypeScript (Vite)
- **Protocol:** Custom JSON over TCP (Port 45679) + UDP Multicast (Port 45678)

---
*Maintained by CashlyCash & HKTITAN.*
*Vibe coded by OpenClaw (orchestrating multiple AI models).*
