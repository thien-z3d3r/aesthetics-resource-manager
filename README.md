# Zen Monitor 🔮

**Zen Monitor** is a high-performance, aesthetically pleasing system monitoring tool designed for **Arch Linux (Hyprland)**, but compatible with all major Linux distributions and Windows 11. Built with **Rust** and `egui`, it features a "Cyberpunk/Glassmorphism" UI with real-time graphs and GPU support.

![Zen Monitor](https://via.placeholder.com/800x400.png?text=Preview+Image+Here)

## Features
*   **Aesthetics:** Transparent, borderless window designed for tiling window managers (Hyprland, Sway) or modern Windows setups.
*   **Performance:** Lightweight (~15MB RAM), written in Rust.
*   **Metrics:** Real-time CPU usage/temps, RAM usage, and NVIDIA GPU stats (via `nvidia-smi`).
*   **Visuals:** Neon gauges, historic plotting, and a custom HUD interface.

## Installation

### Prerequisites
*   **Rust Toolchain:** [Install Rust](https://www.rust-lang.org/tools/install)
*   **NVIDIA Drivers:** Required for GPU stats (uses `nvidia-smi`).

### 🐧 Linux

#### 1. Install Dependencies
**Arch Linux / Manjaro / Hyprland:**
```bash
sudo pacman -S cargo libxcb libxkbcommon openssl
```

**Ubuntu / Debian / Mint:**
You need specific development libraries for the GUI framework (`eframe/egui`).
```bash
sudo apt install cargo libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev libgtk-3-dev
```

**Fedora:**
```bash
sudo dnf install cargo openssl-devel libxcb-devel libxkbcommon-devel gtk3-devel
```

#### 2. Build & Run
```bash
git clone https://github.com/yourusername/zen_monitor.git
cd zen_monitor
cargo build --release
./target/release/zen_monitor
```

### 🪟 Windows 11

1.  **Install Rust:** Download `rustup-init.exe` from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Install Build Tools:** During Rust installation, it will ask for **Visual Studio C++ Build Tools**. Install them.
3.  **Build:**
    Open PowerShell or Command Prompt:
    ```powershell
git clone https://github.com/yourusername/zen_monitor.git
cd zen_monitor
cargo build --release
.	arget\release\zen_monitor.exe
    ```

## Usage

*   **Drag:** Click and drag the top title bar area to move the window.
*   **Close:** Click the small 'X' in the top right corner.
*   **Hyprland Config (Optional):**
    To make it float and look perfect in Hyprland, add this to `~/.config/hypr/hyprland.conf`:
    ```ini
    windowrulev2 = float,class:(zen_monitor)
    windowrulev2 = size 480 420,class:(zen_monitor)
    windowrulev2 = move 100%-490 50,class:(zen_monitor) # Top right corner
    windowrulev2 = opacity 0.9 0.9,class:(zen_monitor)
    ```

## License
MIT License.
