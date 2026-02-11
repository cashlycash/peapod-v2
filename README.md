# PeaPod v2 🫛🚀

> **⚠️ STATUS: ALPHA**
> This software is currently in **ALPHA**. It may crash, eat your bandwidth, or refuse to work. Use at your own risk.

![Test Suite](https://github.com/cashlycash/peapod-v2/actions/workflows/test.yml/badge.svg)

**PeaPod** is a local peer-to-peer swarming protocol. It pools internet connections from nearby devices to speed up downloads.

## 📥 Installation (Alpha)

**Latest Release:** [Check Releases Page](https://github.com/cashlycash/peapod-v2/releases)

### One-Line Installers (Alpha)
*Uses scripts from the repo:*

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
*   **Node.js** (v18+)
*   **Rust** (stable)
*   **Tauri CLI** (`cargo install tauri-cli`)
*   **Linux deps:** `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

### Steps
1.  Clone the repo:
    ```bash
    git clone https://github.com/cashlycash/peapod-v2.git
    cd peapod-v2
    ```
2.  Install frontend dependencies:
    ```bash
    npm install
    ```
3.  Run in Dev Mode:
    ```bash
    npm run tauri dev
    ```
4.  Build Release:
    ```bash
    npm run tauri build
    ```

## 🗺️ Roadmap

1.  **Phase 1: Discovery (✅ Done)**
    *   UDP Multicast beaconing.
    *   React UI for peer listing.
2.  **Phase 2: Transport (🚧 In Progress)**
    *   ✅ TCP Listener established (Port 45679).
    *   ✅ Handshake logic (ID exchange).
    *   🚧 Active Connection (App connecting to discovered peers).
3.  **Phase 3: Chunking (Planned)**
    *   File splitting logic.
4.  **Phase 4: Distribution (Planned)**
    *   Parallel swarm downloads.
5.  **Phase 5: Assembly (Planned)**
    *   Final file reconstruction.

---
*Maintained by CashlyBot & HKTITAN.*
