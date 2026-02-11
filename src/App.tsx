import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import './style.css';

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
    let unlisten: (() => void) | undefined;
    const setupListener = async () => {
      unlisten = await listen<PeerPayload>('peer-update', (event) => {
        const newPeer = event.payload;
        setPeers((prev) => {
          const exists = prev.find((p) => p.id === newPeer.id);
          if (exists) {
            return prev.map((p) => 
              p.id === newPeer.id ? { ...p, lastSeen: Date.now(), status: 'active' } : p
            );
          }
          return [...prev, { ...newPeer, status: 'active', lastSeen: Date.now() }];
        });
      });
    };
    setupListener();
    return () => { if (unlisten) unlisten(); };
  }, []);

  return (
    <div className="layout">
      <header className="header">
        <div className="brand">
          <span className="icon">🫛</span> PEAPOD_V2
        </div>
        <div className="status-badge online">SYSTEM_ONLINE</div>
      </header>
      
      <main className="main-content">
        <section className="panel">
          <div className="panel-header">
            <h3>DISCOVERED_NODES</h3>
            <span className="count">{peers.length}</span>
          </div>
          <div className="panel-body">
            {peers.length === 0 ? (
              <div className="empty-state">
                <span className="scanner">SCANNING_LAN...</span>
              </div>
            ) : (
              <div className="grid">
                {peers.map((peer) => (
                  <div key={peer.id} className="card">
                    <div className="card-header">
                      <span className="peer-name">{peer.name}</span>
                      <span className={`status-indicator ${peer.status}`}></span>
                    </div>
                    <div className="card-body">
                      <div className="stat">
                        <label>ID</label>
                        <span className="mono">{peer.id.slice(0, 8)}...</span>
                      </div>
                      <div className="stat">
                        <label>ADDR</label>
                        <span className="mono">{peer.ip}:{peer.port}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </main>
    </div>
  );
}

export default App;