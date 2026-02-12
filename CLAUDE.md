# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PeaPod v2 is a local peer-to-peer swarming protocol that pools internet connections from nearby devices to speed up downloads. It implements a multi-phase protocol with discovery, transport, chunking, distribution, and assembly capabilities.

## Architecture

The project is structured as a Tauri desktop application with:
- **Frontend**: React + TypeScript (Vite) - UI layer
- **Backend**: Rust (Tokio/Tauri) - Core protocol logic
- **Protocol**: Custom JSON over TCP (Port 45679) + UDP Multicast (Port 45678)

## Key Components

### Rust Backend (src-tauri/src/)
- `main.rs`: Entry point with CLI argument parsing and mode selection (GUI vs daemon)
- `discovery.rs`: UDP multicast discovery implementation
- `transport.rs`: TCP connection handling and message framing
- `protocol.rs`: Message types and serialization
- `chunk.rs`: File chunking and management
- `state.rs`: Application state management

### Frontend (src/)
- `main.tsx`: React application entry point
- `App.tsx`: Main application component with UI
- `style.css`: Styling

## Development Setup

### Prerequisites
- Node.js (v20+)
- Rust (stable)
- Tauri CLI (`cargo install tauri-cli`)
- Linux dependencies: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `libssl-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`

### Commands
- `npm install`: Install frontend dependencies
- `npm run tauri dev`: Run in development mode with GUI
- `npm run build`: Build the frontend
- `npm run tauri build`: Build the full desktop application
- `cd src-tauri && cargo test`: Run backend tests

## Key Features

### Modes
- **GUI Mode**: Full desktop application with UI
- **Daemon Mode**: Headless operation (`--daemon` flag) for servers or environments with EGL errors

### Protocol Phases
1. **Discovery**: UDP multicast beaconing on 239.255.60.60:45678
2. **Transport**: TCP connections on port 45679 with 4-byte length header framing
3. **Chunking**: File splitting and state tracking
4. **Distribution**: Requesting and serving chunks
5. **Assembly**: Writing chunks to disk

## Important Files

- `README.md`: Project overview and usage instructions
- `PROTOCOL.md`: Protocol specification
- `src-tauri/src/main.rs`: Main application logic and mode handling
- `src-tauri/src/discovery.rs`: Discovery protocol implementation
- `src-tauri/src/transport.rs`: Transport protocol implementation
- `src-tauri/src/chunk.rs`: Chunking logic