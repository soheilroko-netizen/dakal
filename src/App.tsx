import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import './App.css';

interface StatusInfo {
  running: boolean;
  ping_ms: number | null;
  uptime_secs: number | null;
  server_ip: string;
  server_port: number;
}

type Page = 'dashboard' | 'profiles' | 'settings';

const sidebarIcons: { page: Page; icon: string }[] = [
  { page: 'dashboard', icon: '🛡️' },
  { page: 'profiles', icon: '🌍' },
  { page: 'settings', icon: '⚙️' },
];

const bottomIcons = ['⚡', '🔒', '📶', '∞', '✦'];

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

function App() {
  const [page, setPage] = useState<Page>('dashboard');
  const [status, setStatus] = useState<StatusInfo>({
    running: false,
    ping_ms: null,
    uptime_secs: null,
    server_ip: '—',
    server_port: 0,
  });
  const [connecting, setConnecting] = useState(false);
  const [showConfigs, setShowConfigs] = useState(false);
  const [profiles, setProfiles] = useState<string[]>([]);
  const [activeProfile, setActiveProfile] = useState('default');

  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<StatusInfo>('get_status');
      setStatus(s);
    } catch (e) {
      // not running
    }
  }, []);

  const fetchProfiles = useCallback(async () => {
    try {
      const p = await invoke<string[]>('list_configs');
      setProfiles(p);
    } catch (_) {}
  }, []);

  useEffect(() => {
    fetchStatus();
    fetchProfiles();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus, fetchProfiles]);

  const toggleVPN = async () => {
    setConnecting(true);
    try {
      if (status.running) {
        await invoke('stop_vpn');
      } else {
        await invoke('start_vpn');
      }
      // wait a moment then refresh
      setTimeout(fetchStatus, 2000);
    } catch (e) {
      alert(e);
    }
    setConnecting(false);
  };

  const selectProfile = async (name: string) => {
    try {
      await invoke('load_profile', { name });
      setActiveProfile(name);
      setShowConfigs(false);
    } catch (e) {
      alert(e);
    }
  };

  const pingClass = (ms: number | null) => {
    if (ms === null) return '';
    if (ms < 100) return 'ping-good';
    if (ms < 200) return 'ping-ok';
    return 'ping-bad';
  };

  const win = getCurrentWindow();

  const renderDashboard = () => (
    <div className="content">
      {/* Status Pill */}
      <div className="status-pill">
        <span className={`status-dot ${status.running ? 'connected' : ''}`} />
        {status.running ? (
          <span>{status.server_ip ? `🌍 ${status.server_ip}` : 'Connected'}</span>
        ) : (
          <span>Disconnected</span>
        )}
      </div>

      {/* Planet Button */}
      <div className="planet-container" onClick={toggleVPN}>
        <div className="planet-ring" />
        <div className="planet-ring" />
        <div
          className={`planet ${status.running ? 'connected' : ''}`}
          style={connecting ? { opacity: 0.6 } : {}}
        >
          <span className="planet-icon">
            {status.running ? '⏻' : '⏻'}
          </span>
        </div>
      </div>
      <span className="planet-label">
        {connecting ? 'Connecting...' : status.running ? 'Connected' : 'Tap to Connect'}
      </span>

      {/* Stats */}
      <div className="stats-row">
        <div className="stat-card">
          <div className={`stat-value ${pingClass(status.ping_ms)}`}>
            {status.ping_ms !== null ? `${status.ping_ms}ms` : '—'}
          </div>
          <div className="stat-label">Ping</div>
        </div>
        <div className="stat-card">
          <div className={`stat-value ${status.running ? 'connected-text' : ''}`}>
            {status.uptime_secs !== null ? formatUptime(status.uptime_secs) : '—'}
          </div>
          <div className="stat-label">Uptime</div>
        </div>
        <div className="stat-card" onClick={() => setShowConfigs(!showConfigs)} style={{ cursor: 'pointer' }}>
          <div className="stat-value" style={{ fontSize: 16 }}>
            {activeProfile}
          </div>
          <div className="stat-label">Profile</div>
        </div>
      </div>

      {/* Profile selector popup */}
      {showConfigs && (
        <div className="config-panel">
          <h3>Profiles</h3>
          {profiles.map((name) => (
            <div
              key={name}
              className={`profile-item ${name === activeProfile ? 'active' : ''}`}
              onClick={() => selectProfile(name)}
            >
              <span>{name}</span>
              {name === activeProfile && <span className="check">✓</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  );

  return (
    <div className="app-shell">
      {/* Title Bar */}
      <div className="titlebar">
        <div className="titlebar-left">
          <span className="titlebar-title">DAKAL</span>
        </div>
        <div className="titlebar-controls">
          <button className="titlebar-btn minimize" onClick={() => win.minimize()} />
          <button className="titlebar-btn maximize" onClick={() => win.toggleMaximize()} />
          <button className="titlebar-btn close" onClick={() => win.close()} />
        </div>
      </div>

      {/* Main Area */}
      <div className="main-area">
        {/* Sidebar */}
        <div className="sidebar">
          {sidebarIcons.map(({ page: p, icon }) => (
            <button
              key={p}
              className={`sidebar-btn ${page === p ? 'active' : ''}`}
              onClick={() => setPage(p)}
              title={p}
            >
              {icon}
            </button>
          ))}
          <div className="sidebar-spacer" />
        </div>

        {/* Content */}
        {page === 'dashboard' && renderDashboard()}
        {page === 'profiles' && (
          <div className="content" style={{ justifyContent: 'flex-start', paddingTop: 40 }}>
            <h3 style={{ color: 'var(--text-secondary)', fontSize: 13, marginBottom: 12, textTransform: 'uppercase', letterSpacing: 1 }}>
              Config Profiles
            </h3>
            {profiles.map((name) => (
              <div
                key={name}
                className={`profile-item ${name === activeProfile ? 'active' : ''}`}
                onClick={() => selectProfile(name)}
                style={{ width: '100%' }}
              >
                <span>{name}</span>
                {name === activeProfile && <span className="check">✓</span>}
              </div>
            ))}
          </div>
        )}
        {page === 'settings' && (
          <div className="content">
            <div style={{ color: 'var(--text-secondary)', fontSize: 12, textAlign: 'center' }}>
              Settings
              <div style={{ marginTop: 8, fontSize: 11, color: 'var(--text-muted)' }}>
                More options coming soon
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Bottom Bar */}
      <div className="bottombar">
        {bottomIcons.map((icon, i) => (
          <button
            key={i}
            className={`bottombar-btn ${i === 0 ? 'active' : ''}`}
          >
            {icon}
          </button>
        ))}
      </div>
    </div>
  );
}

export default App;
