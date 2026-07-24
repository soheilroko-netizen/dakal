import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const app = document.getElementById('app')!;

let state = {
  connected: false,
  statusText: 'Disconnected',
  ping: 0,
  uptime: '',
  tx: 0,
  rx: 0,
  error: '',
};

function render() {
  app.innerHTML = `
    <div class="container">
      <div class="planet-btn ${state.connected ? 'connected' : ''}" id="connectBtn">
        <svg class="power-icon" viewBox="0 0 64 64" fill="none">
          <path d="M32 8v24" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
          <path d="M20 18a20 20 0 1 0 24 0" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
        </svg>
        <div class="ring"></div>
      </div>
      <div class="status">${state.statusText}</div>
      ${state.error ? `<div class="error">${state.error}</div>` : ''}
      <div class="stats">
        <div class="stat">Ping: ${state.ping}ms</div>
        <div class="stat">${state.uptime}</div>
      </div>
    </div>
  `;

  document.getElementById('connectBtn')?.addEventListener('click', async () => {
    try {
      state.error = '';
      if (state.connected) {
        await invoke('stop_vpn');
      } else {
        await invoke('start_vpn');
      }
      // Request status update after action
      const status = await invoke<any>('get_status');
      state.connected = status.running;
      state.statusText = status.running ? 'Connected' : 'Disconnected';
      state.ping = status.ping || 0;
      render();
    } catch (e: any) {
      state.error = typeof e === 'string' ? e : e?.message || 'Connection failed';
      render();
    }
  });
}

render();

// Listen for status updates
listen<any>('vpn-status', (e) => {
  const s = e.payload;
  state.connected = s.running;
  state.statusText = s.running ? 'Connected' : 'Disconnected';
  if (s.ping !== undefined) state.ping = s.ping;
  if (s.uptime !== undefined) state.uptime = s.uptime;
  state.error = '';
  render();
});
