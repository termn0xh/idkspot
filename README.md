# idkspot

A native GTK4 Linux app for Wi-Fi Hotspot creation.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![GTK4](https://img.shields.io/badge/GTK4-4A86CF?style=flat&logo=gnome&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black)

## Features

- **Hardware Check** — Detects if your Wi-Fi card supports AP+Managed mode
- **Auto-Detection** — Finds wireless interface and current channel
- **One-Click Hotspot** — Start/Stop with a single button
- **System Tray** — Minimizes to tray, persists in background

## Dependencies

| Package | Purpose |
|---------|---------|
| `linux-wifi-hotspot` | Provides `create_ap` command |
| `gtk4` | GUI framework |
| `libadwaita` | GNOME styling |
| `dbus` | System tray communication |
| `iw` | Wireless interface detection |
| `polkit` | Privilege elevation (`pkexec`) |

### Arch Linux

```bash
sudo pacman -S linux-wifi-hotspot gtk4 libadwaita dbus iw polkit
```

### Debian/Ubuntu

```bash
sudo apt install create-ap libgtk-4-1 libadwaita-1-0 libdbus-1-dev iw policykit-1
```

## Installation

```bash
# Clone
git clone https://github.com/termn0xh/idkspot.git
cd idkspot

# Build
cargo build --release

# Install binary
sudo cp target/release/idkspot /usr/bin/

# Install desktop entry (shows in app menu)
sudo cp idkspot.desktop /usr/share/applications/
```

## Usage

```bash
idkspot
```

Or search for **idkspot** in your application menu.

1. App checks hardware compatibility on startup
2. Enter SSID and password (min 8 chars)
3. Click **Start Hotspot**
4. Close window → App minimizes to system tray
5. Click tray icon to reopen, right-click for menu

## License

MIT
