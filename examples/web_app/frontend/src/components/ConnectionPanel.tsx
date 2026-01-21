import React, { useState } from 'react';
import './ConnectionPanel.css';
import { ConnectionStatus } from '../types';

interface ConnectionPanelProps {
  onConnect: (address: string) => void;
  onDisconnect: () => void;
  connectionStatus: ConnectionStatus;
  loading: boolean;
}

const ConnectionPanel: React.FC<ConnectionPanelProps> = ({
  onConnect,
  onDisconnect,
  connectionStatus,
  loading,
}) => {
  const [address, setAddress] = useState('192.168.1.120:44818');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (address.trim()) {
      onConnect(address.trim());
    }
  };

  return (
    <div className="connection-panel">
      <h2>🔌 Connection</h2>
      {!connectionStatus.connected ? (
        <form onSubmit={handleSubmit} className="connection-form">
          <div className="form-group">
            <label htmlFor="address">PLC Address:</label>
            <input
              id="address"
              type="text"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="192.168.1.120:44818"
              disabled={loading}
              required
            />
          </div>
          <button type="submit" disabled={loading} className="btn btn-primary">
            {loading ? 'Connecting...' : 'Connect'}
          </button>
        </form>
      ) : (
        <div className="connected-state">
          <div className="status-indicator connected">
            <span className="status-dot"></span>
            Connected to {connectionStatus.address}
          </div>
          <button
            onClick={onDisconnect}
            disabled={loading}
            className="btn btn-danger"
          >
            {loading ? 'Disconnecting...' : 'Disconnect'}
          </button>
        </div>
      )}
    </div>
  );
};

export default ConnectionPanel;

