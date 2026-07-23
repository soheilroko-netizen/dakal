# Dakal VPN

Modern Windows VPN client with sing-box + ShadowTLS. Space-themed UI.

## Features
- Launch sing-box with one-click
- Real-time ping & uptime display
- System tray (minimize to tray)
- Multi-config profile manager
- Auto admin elevation

## Build
```bash
npm install
npm run build       # frontend
cargo tauri build   # creates MSI installer
```

## Config
Place `sing-box.exe` and `config.json` next to `dakal.exe`.
Profiles saved in `profiles/` folder.
