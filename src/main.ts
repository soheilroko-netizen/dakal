import './styles.css';

const app = document.getElementById('app')!;

let state = {
  connected: false,
  statusText: 'Disconnected',
  ping: 0,
  uptime: '',
  tx: 0,
  rx: 0,
};

function render() {
  app.innerHTML = `
    <div class="container">
      <div class="planet-btn" id="connectBtn">
        <svg class="power-icon" viewBox="0 0 64 64" fill="none">
          <path d="M32 8v24" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
          <path d="M20 18a20 20 0 1 0 24 0" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
        </svg>
        <div class="ring"></div>
      </div>
      <div class="status">${state.statusText}</div>
      <div class="stats">
        <div class="stat">Ping: ${state.ping}ms</div>
        <div class="stat">${state.uptime}</div>
      </div>
    </div>
  `;

  document.getElementById('connectBtn')?.addEventListener('click', () => {
    const { invoke } = (window as any).__TAURI__?.core || {};
    if (!invoke) return;
    if (state.connected) {
      invoke('stop_vpn');
    } else {
      invoke('start_vpn');
    }
  });
}

render();

// Listen for status updates
if ((window as any).__TAURI__) {
  const { listen } = (window as any).__TAURI__.event;
  listen('vpn-status', (e: any) => {
    const s = e.payload;
    state.connected = s.running;
    state.statusText = s.running ? 'Connected' : 'Disconnected';
    state.ping = s.ping || 0;
    state.uptime = s.uptime || '';
    render();
  });
  listen('vpn-ping', (e: any) => {
    state.ping = e.payload;
  });
}
