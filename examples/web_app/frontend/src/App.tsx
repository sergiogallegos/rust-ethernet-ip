import React, { useState, useEffect } from 'react';
import './App.css';
import ConnectionPanel from './components/ConnectionPanel';
import TagOperations from './components/TagOperations';
import StatusBar from './components/StatusBar';
import { ConnectionStatus, PlcValueJson } from './types';

const API_BASE = process.env.REACT_APP_API_URL || 'http://localhost:3000';

function App() {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
    connected: false,
    address: null,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check connection status on mount and periodically
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const response = await fetch(`${API_BASE}/api/status`);
        const data = await response.json();
        setConnectionStatus({
          connected: data.connected,
          address: data.address,
        });
      } catch (err) {
        console.error('Failed to check status:', err);
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 5000); // Check every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const handleConnect = async (address: string) => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE}/api/connect`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ address }),
      });

      const data = await response.json();
      if (data.success) {
        setConnectionStatus({
          connected: true,
          address: address,
        });
      } else {
        setError(data.error || 'Failed to connect');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Connection failed');
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnect = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE}/api/disconnect`, {
        method: 'POST',
      });

      const data = await response.json();
      if (data.success) {
        setConnectionStatus({
          connected: false,
          address: null,
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Disconnect failed');
    } finally {
      setLoading(false);
    }
  };

  const handleReadTag = async (tagName: string): Promise<{ value: PlcValueJson | null; dataType: string | null; error: string | null }> => {
    try {
      const response = await fetch(`${API_BASE}/api/read`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ tag_name: tagName }),
      });

      const data = await response.json();
      if (data.success) {
        return {
          value: data.value,
          dataType: data.data_type,
          error: null,
        };
      } else {
        return {
          value: null,
          dataType: null,
          error: data.error || 'Failed to read tag',
        };
      }
    } catch (err) {
      return {
        value: null,
        dataType: null,
        error: err instanceof Error ? err.message : 'Read failed',
      };
    }
  };

  const handleWriteTag = async (
    tagName: string,
    value: PlcValueJson
  ): Promise<{ success: boolean; error: string | null }> => {
    try {
      const response = await fetch(`${API_BASE}/api/write`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          tag_name: tagName,
          value: value,
        }),
      });

      const data = await response.json();
      return {
        success: data.success,
        error: data.error || null,
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : 'Write failed',
      };
    }
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>🦀 PLC Web Interface</h1>
        <p>Rust Backend + React/TypeScript Frontend</p>
      </header>

      <main className="App-main">
        <ConnectionPanel
          onConnect={handleConnect}
          onDisconnect={handleDisconnect}
          connectionStatus={connectionStatus}
          loading={loading}
        />

        {error && (
          <div className="error-message">
            <span>⚠️</span> {error}
          </div>
        )}

        {connectionStatus.connected && (
          <TagOperations
            onReadTag={handleReadTag}
            onWriteTag={handleWriteTag}
          />
        )}

        <StatusBar connectionStatus={connectionStatus} />
      </main>
    </div>
  );
}

export default App;

