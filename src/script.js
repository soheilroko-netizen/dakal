const { invoke } = window.__TAURI__.invoke;
const { appWindow } = window.__TAURI__.window;

let powerBtn = document.getElementById('power');
let running = false;

async function toggleVpn() {
  if (running) {
    await invoke('stop_vpn');
    running = false;
  } else {
    await invoke('start_vpn');
    running = true;
  }
  updateButton();
}

function updateButton() {
  if (running) {
    powerBtn.classList.remove('off');
    powerBtn.classList.add('on');
  } else {
    powerBtn.classList.remove('on');
    powerBtn.classList.add('off');
  }
}

// window controls
document.getElementById('minimize').addEventListener('click', async () => {
  await appWindow.minimize();
});

document.getElementById('maximize').addEventListener('click', async () => {
  await appWindow.toggleMaximize();
});

document.getElementById('close').addEventListener('click', async () => {
  await appWindow.close();
});

powerBtn.addEventListener('click', toggleVpn);