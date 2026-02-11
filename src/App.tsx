import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Peer {
  id: string;
  name: string;
  ip: string;
  status: 'active' | 'inactive';
}

function App() {
  const [peers, setPeers] = useState<Peer[]>([]);

  useEffect(() => {
    // Listen for discovery events (placeholder)
    const unlisten = async () => {
      // await listen<Peer>('peer-update', (event) => {
      //   setPeers((prev) => [...prev, event.payload]);
      // });
    };
    unlisten();
  }, []);

  return (
    <div className="container">
      <h1>PeaPod Discovery</h1>
      
      <div className="peers-list">
        {peers.length === 0 ? (
          <p>Scanning for nearby pods...</p>
        ) : (
          <ul>
            {peers.map((peer) => (
              <li key={peer.id} className="peer-item">
                <span className="peer-name">{peer.name}</span>
                <span className="peer-ip">({peer.ip})</span>
                <span className={`status-dot ${peer.status}`}></span>
              </li>
            ))}
          </ul>
        )}
      </div>

      <button onClick={() => invoke('start_scan')}>Rescan Network</button>
    </div>
  );
}

export default App;