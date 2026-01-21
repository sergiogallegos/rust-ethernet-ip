import React from 'react';
import './StatusBar.css';
import { ConnectionStatus } from '../types';

interface StatusBarProps {
  connectionStatus: ConnectionStatus;
}

const StatusBar: React.FC<StatusBarProps> = ({ connectionStatus }) => {
  return (
    <div className="status-bar">
      <div className="status-item">
        <span className="status-label">Status:</span>
        <span
          className={`status-value ${
            connectionStatus.connected ? 'connected' : 'disconnected'
          }`}
        >
          {connectionStatus.connected ? '🟢 Connected' : '🔴 Disconnected'}
        </span>
      </div>
      {connectionStatus.address && (
        <div className="status-item">
          <span className="status-label">Address:</span>
          <span className="status-value">{connectionStatus.address}</span>
        </div>
      )}
    </div>
  );
};

export default StatusBar;

