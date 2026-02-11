import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface PeerPayload {
  id: string;
  name: string;
  ip: string;
  port: number;
}

interface ConnectionEvent {
  peer_id: string;
  status: 'connected' | 'disconnected' | 'error';
}

interface Peer extends PeerPayload {
  status: 'active' | 'inactive';
  lastSeen: number;
  connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'error';
}

function App() {
  const [peers, setPeers] = useState<Peer[]>([]);

  useEffect(() => {
    let unlistenPeer: (() => void) | undefined;
    let unlistenConn: (() => void) | undefined;

    const setupListeners = async () => {
      // Listen for discovery events from Rust
      unlistenPeer = await listen<PeerPayload>('peer-update', (event) => {
        const newPeer = event.payload;
        setPeers((prev) => {
          const exists = prev.find((p) => p.id === newPeer.id);
          if (exists) {
            return prev.map((p) => 
              p.id === newPeer.id ? { ...p, lastSeen: Date.now(), status: 'active' } : p
            );
          }
          return [...prev, { ...newPeer, status: 'active', lastSeen: Date.now(), connectionStatus: 'disconnected' }];
        });
      });

      // Listen for connection status events
      unlistenConn = await listen<ConnectionEvent>('connection-status', (event) => {
        const { peer_id, status } = event.payload;
        setPeers((prev) =>
          prev.map((p) =>
            p.id === peer_id
              ? { ...p, connectionStatus: status === 'connected' ? 'connected' : status === 'error' ? 'error' : 'disconnected' }
              : p
          )
        );
      });
    };

    setupListeners();

    return () => {
      if (unlistenPeer) unlistenPeer();
      if (unlistenConn) unlistenConn();
    };
  }, []);

  const handleConnect = async (peer: Peer) => {
    setPeers((prev) =>
      prev.map((p) => (p.id === peer.id ? { ...p, connectionStatus: 'connecting' } : p))
    );

    try {
      await invoke('connect_to_peer', {
        peerId: peer.id,
        ip: peer.ip,
        port: peer.port,
      });
    } catch (err) {
      console.error('Connection failed:', err);
      setPeers((prev) =>
        prev.map((p) => (p.id === peer.id ? { ...p, connectionStatus: 'error' } : p))
      );
    }
  };

  const handleDisconnect = async (peer: Peer) => {
    try {
      await invoke('disconnect_from_peer', { peerId: peer.id });
    } catch (err) {
      console.error('Disconnect failed:', err);
    }
  };

  return (
    <div className="container">
      <h1>PeaPod Discovery 🫛</h1>
      <p className="subtitle">Scanning for nearby pods...</p>
      
      <div className="peers-list">
        {peers.length === 0 ? (
          <div className="empty-state">
            <p>No peers found yet.</p>
            <div className="pulse-loader"></div>
          </div>
        ) : (
          <ul>
            {peers.map((peer) => (
              <li key={peer.id} className="peer-item">
                <div className="peer-info">
                  <span className="peer-name">{peer.name}</span>
                  <span className="peer-ip">{peer.ip}:{peer.port}</span>
                </div>
                <div className="peer-status">
                  <span className={`status-dot ${peer.status}`}></span>
                  <span className="status-text">{peer.status}</span>
                </div>
                <div className="peer-actions">
                  {peer.connectionStatus === 'connected' ? (
                    <button className="btn-disconnect" onClick={() => handleDisconnect(peer)}>
                      Disconnect
                    </button>
                  ) : peer.connectionStatus === 'connecting' ? (
                    <button className="btn-connecting" disabled>
                      Connecting...
                    </button>
                  ) : (
                    <button className="btn-connect" onClick={() => handleConnect(peer)}>
                      Connect
                    </button>
                  )}
                  {peer.connectionStatus === 'error' && (
                    <span className="connection-error">Connection failed</span>
                  )}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="actions">
        <button onClick={() => setPeers([])}>Clear List</button>
      </div>
    </div>
  );
}

export default App;