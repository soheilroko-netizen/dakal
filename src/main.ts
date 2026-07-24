import { invoke } from '@tauri-apps/api/core';

let running = false;

const btn = document.getElementById('connect-btn')!;
const statusText = document.getElementById('status-text')!;
const statusDot = document.getElementById('status-dot')!;
const pingValue = document.getElementById('ping-value')!;

function setConnected() {
  running = true;
  btn.textContent = 'Disconnect';
  btn.classList.add('connected');
  statusText.textContent = 'Connected';
  statusDot.className = 'dot green';
}

function setDisconnected() {
  running = false;
  btn.textContent = 'Connect';
  btn.classList.remove('connected');
  statusText.textContent = 'Disconnected';
  statusDot.className = 'dot red';
  pingValue.textContent = '-- ms';
}

async function toggle() {
  if (running) {
    await invoke('disconnect');
    setDisconnected();
  } else {
    btn.textContent = 'Connecting...';
    try {
      await invoke('connect');
      setConnected();
    } catch (e) {
      statusText.textContent = `Error: ${e}`;
      btn.textContent = 'Connect';
    }
  }
}

async function poll() {
  try {
    const status: any = await invoke('get_status');
    if (status.running) {
      if (!running) setConnected();
      pingValue.textContent = status.ping > 0 ? `${status.ping} ms` : '-- ms';
    } else {
      if (running) setDisconnected();
    }
  } catch {}
}

btn.addEventListener('click', toggle);
setInterval(poll, 2000);
poll();
