import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface PeerPayload {
  id: string;
  name: string;
  ip: string;
  port: number;
}

interface Peer extends PeerPayload {
  status: 'active' | 'inactive';
  lastSeen: number;
}

function App() {
  const [peers, setPeers] = useState<Peer[]>([]);

  useEffect(() => {
    // Listen for discovery events from Rust
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<PeerPayload>('peer-update', (event) => {
        const newPeer = event.payload;
        setPeers((prev) => {
          const exists = prev.find((p) => p.id === newPeer.id);
          if (exists) {
            // Update timestamp
            return prev.map((p) => 
              p.id === newPeer.id ? { ...p, lastSeen: Date.now(), status: 'active' } : p
            );
          }
          // Add new peer
          return [...prev, { ...newPeer, status: 'active', lastSeen: Date.now() }];
        });
      });
    };

    setupListener();

    // Cleanup on unmount
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

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