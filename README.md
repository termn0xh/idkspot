# idkspot

A native Rust GUI for Linux Wi-Fi Hotspots (Simultaneous Mode).

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black)

## Features

- **Hardware Check** — Automatically detects if your Wi-Fi card supports simultaneous AP+Managed mode
- **Auto-Detection** — Finds your wireless interface and current channel
- **One-Click Hotspot** — Start a hotspot with the IGNITE button
- **Clean Stop** — Gracefully stop the hotspot with the STOP button
- **Modern UI** — Dark theme with cyan accents

## Dependencies

| Package | Purpose |
|---------|---------|
| `linux-wifi-hotspot` | Provides `create_ap` command |
| `polkit` | For `pkexec` privilege elevation |
| `gcc` | Rust compilation |
| `pkg-config` | Build dependency resolution |
| `iw` | Wireless interface detection |

### Install Dependencies (Arch Linux)

```bash
sudo pacman -S linux-wifi-hotspot polkit gcc pkg-config iw
```

### Install Dependencies (Debian/Ubuntu)

```bash
sudo apt install create-ap policykit-1 gcc pkg-config iw
```

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/idkspot.git
cd idkspot

# Build release binary
cargo build --release

# (Optional) Install system-wide
sudo cp target/release/idkspot /usr/bin/
```

## Usage

```bash
# Run from build directory
./target/release/idkspot

# Or if installed system-wide
idkspot
```

1. The app checks hardware compatibility on startup
2. Enter your desired SSID and password (min 8 characters)
3. Click **IGNITE** to start the hotspot
4. Click **STOP** to stop the hotspot

## License

MIT
