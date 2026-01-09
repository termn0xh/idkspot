# idkspot

A native GTK4 Linux application for Wi-Fi Hotspot creation using simultaneous AP mode.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![GTK4](https://img.shields.io/badge/GTK4-4A86CF?style=flat&logo=gnome&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black)

## Features

- Hardware compatibility detection for AP+Managed simultaneous mode
- Automatic wireless interface and channel detection
- One-click hotspot start/stop
- Connected devices dialog with hostname lookup
- System tray integration with background operation
- Single-instance application behavior
- Native GNOME/Adwaita theming

## Dependencies

| Package | Purpose |
|---------|---------|
| `linux-wifi-hotspot` | Provides `create_ap` command |
| `gtk4` | GUI framework |
| `libadwaita` | GNOME styling |
| `dbus` | System tray communication |
| `iw` | Wireless interface detection |
| `polkit` | Privilege elevation |

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
git clone https://github.com/yourusername/idkspot.git
cd idkspot

cargo build --release

sudo cp target/release/idkspot /usr/bin/
sudo cp idkspot.desktop /usr/share/applications/
```

## Usage

```bash
idkspot
```

Or launch from the application menu.

1. The application checks hardware compatibility on startup
2. Enter SSID and password (minimum 8 characters)
3. Click Start Hotspot (authentication required for create_ap)
4. View connected devices via the Devices button
5. Close window to minimize to system tray

## License

MIT
