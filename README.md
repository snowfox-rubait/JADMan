# JADMan — Just Another Download Manager

> The download manager Linux deserves. A full IDM replacement — and then some.

JADMan is a keyboard-driven, terminal-native download manager and stream interceptor, built in Rust.
It handles everything: segmented file downloads, video/audio from thousands of sites,
scheduled triggers, browser integration, and a fast TUI — all orchestrated by a persistent
background daemon.

No Electron. No GUI framework. No mouse required.

---

## Why JADMan?

Internet Download Manager (IDM) is the gold standard on Windows — and Linux has never had
a real answer to it. JADMan is that answer, and it goes further:

| Feature | IDM | JADMan |
|---|:---:|:---:|
| Segmented / multi-connection downloads | ✅ | ✅ |
| Browser integration (intercept clicks) | ✅ | ✅ |
| Schedule downloads | ✅ | ✅ |
| Video site downloads (YouTube, etc.) | ❌ | ✅ |
| Media-aware triggers (start based on conditions) | ❌ | ✅ |
| Terminal-native TUI | ❌ | ✅ |
| Inline file preview (images, video, code) | ❌ | ✅ |
| Clipboard monitor | ✅ | ✅ |
| Batch download with smart link extraction | ✅ | ✅ |
| Fetch interception (catches XHR/fetch streams) | ❌ | ✅ |
| Proxy / Tor support | ✅ | ✅ |
| Fully open source, forever | ❌ | ✅ |

---

## Features

### Core Engine
- **Segmented downloads** via `aria2c` — split files into up to 16 parallel connections
- **Video & audio downloads** via `yt-dlp` — supports YouTube, Twitter/X, Twitch, and
  [thousands more](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)
- **Smart URL routing** — automatically decides whether a URL goes to `aria2c` or `yt-dlp`
- **Post-processing hooks** via `ffmpeg` — auto-remux, compress, or convert after download

### Scheduler & Triggers
Schedule any download to start based on real conditions, not just a clock:

- **Time** — start at a specific date/time
- **On complete** — chain downloads (start B when A finishes)
- **Storage threshold** — start when disk space drops below N GB (Planned)
- **Media position** — start when you've watched 75% of an episode (Planned)

### Browser Extension (Chrome / Firefox)
A 5-layer capture system that catches downloads other extensions miss:

- Intercepts Chrome's native download dialog and reroutes to JADMan
- Sniffs network responses by MIME type and file size
- Injects download buttons directly onto `<video>` and `<audio>` elements on any page
- Patches `fetch` and `XMLHttpRequest` at page level to catch streams and blobs
- Batch grabber UI for bulk-selecting and downloading multiple links at once

### TUI (Terminal UI)
Built with `ratatui`. Three-panel layout: categories, download list, detail/preview.

```
┌─────────────────┬──────────────────────────────┬───────────────────────────┐
│   Categories    │         Download List         │     Detail / Preview      │
│                 │                               │                           │
│ ▶ Unfinished    │  archlinux.iso   2.7G  4.2M/s │  [########c      ]  45%  │
│   Videos        │  ubuntu.iso      1.1G    —    │                           │
│   Audio         │  paper.pdf       1.1M    —    │  Size       2.70 GB      │
│   Documents     │  song.mp3        8.2M  1.1M/s │  Rate       4.2 MB/s     │
│   Programs      │                               │  ETA        6m 32s       │
│                 │                               │  Engine     aria2c        │
│ ▶ Finished      │                               │                           │
│   ...           │                               │  [preview area]           │
└─────────────────┴──────────────────────────────┴───────────────────────────┘
```

- **Pacman-style progress bar** — `[########c    ]` with a moving mouth character
- **Inline preview** — syntax-highlighted code/text via `syntect`, images/video via `ffmpeg` hooks
- **Fully keyboard-driven** — vim-style navigation, submenu keybinds, no mouse needed
- **Fuzzy search**, visual range select, sort by any column

### Everything Else
- **Clipboard monitor** — watches your clipboard for URLs and offers to download them
- **Notifications** via `libnotify` (`notify-rust`)
- **Antivirus hook** — optional post-download `clamscan`
- **Proxy / Tor support** — SOCKS5 passthrough
- **SQLite database** — persistent queue, survives daemon restarts
- **TOML config** — sane defaults, fully configurable

---

## Architecture

JADMan runs as a background daemon (`jadm-daemon`) that everything else connects to.
The TUI and browser extension are just clients — the daemon is the brain.

```
Browser Extension  ──HTTP (localhost:6246)──▶  jadm-daemon
                                                    │
TUI (jadm-tui)     ──Unix socket──────────────────▶ │
                                                    │
                                             aria2c / yt-dlp / ffmpeg
                                             SQLite
                                             Scheduler
                                             Clipboard monitor
```

---

## Installation & Setup

JADMan is fully cross-platform and runs natively on **Linux**, **Windows**, and **macOS**. 

### 1. Install Core Dependencies
JADMan orchestrates high-speed downloads and stream stitching using `aria2`, `yt-dlp`, and `ffmpeg`. Install them for your platform:

*   **Windows**: Open PowerShell and run:
    ```powershell
    winget install aria2.aria2 yt-dlp.yt-dlp Gyan.FFmpeg
    ```
*   **macOS**: Open Terminal and run:
    ```bash
    brew install aria2 yt-dlp ffmpeg
    ```
*   **Linux**: Install via your package manager:
    ```bash
    sudo pacman -S aria2 yt-dlp ffmpeg   # Arch Linux
    sudo apt install aria2 yt-dlp ffmpeg # Debian/Ubuntu
    ```

### 2. Get JADMan
You can build JADMan from source (requires the Rust toolchain):
```bash
git clone https://codeberg.org/snowfox-rubait-96/jadman.git
cd jadman
cargo build --release
```
*   The compiled daemon will be at `target/release/jadm-daemon` (or `jadm-daemon.exe`).
*   The TUI client will be at `target/release/jadm-tui` (or `jadm-tui.exe`).

### 3. Register Browser Native Messaging Host
To allow browser extensions to send downloads to JADMan, register the manifest:
*   **Windows**: Open CMD/PowerShell as user in the folder containing `jadm-daemon.exe` and run:
    ```cmd
    jadm-daemon.exe install-native-manifest
    ```
*   **Linux / macOS**: Open terminal in the build output folder and run:
    ```bash
    ./jadm-daemon install-native-manifest
    ```

### 4. Install Browser Extension
1.  Open your browser (Chrome/Brave/Edge) and go to `chrome://extensions`.
2.  Enable **Developer mode** (toggle switch in the top-right corner).
3.  Click **Load unpacked** (top-left) and select the `extension/chrome/` folder inside the repository.
*(For Firefox, go to `about:debugging#/runtime/this-firefox`, click **Load Temporary Add-on**, and select any file inside `extension/firefox/`)*

---

## Usage

1.  **Start the background daemon**:
    *   **Linux / macOS**: `jadm-daemon &` (or run in background/systemd).
    *   **Windows**: Launch `jadm-daemon.exe` (runs in background shell).
2.  **Open TUI Console**:
    *   Run `jadm-tui` (or `jadm-tui.exe` on Windows) to view the queue and control downloads via keyboard shortcuts.
```

---

## Keybindings (TUI)

| Key | Action |
|---|---|
| `h / l` | Move focus between panels |
| `j / k` | Move up / down |
| `a` | Add URL |
| `A` | Batch add |
| `p` | Pause selected |
| `r` | Resume selected |
| `s` | Stop selected |
| `d` | Remove from list |
| `D` | Remove + delete file |
| `o` | Open file |
| `O` | Open folder |
| `t` | Set schedule / trigger |
| `/` | Fuzzy search |
| `?` | Help overlay |
| `q` | Quit |

---

## Configuration

Config lives at `~/.config/jadm/config.toml`. A default template is generated on first run.

```toml
[general]
default_folder = "~/Downloads"
max_connections = 8
speed_limit = 0          # 0 = unlimited

[folders]
Videos     = "~/Downloads/Videos"
Audio      = "~/Downloads/Audio"
Documents  = "~/Downloads/Documents"
Programs   = "~/Downloads/Programs"

[ytdlp]
format = "bestvideo+bestaudio/best"
embed_subs = true

[ffmpeg]
auto_remux = false
auto_compress = false
```

---

## License

JADMan is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

See [`LICENSE`](LICENSE) for the full license text.

---

## Roadmap

- [x] Daemon architecture + aria2c integration
- [x] yt-dlp subprocess runner + URL detection
- [x] Download queue + SQLite persistence
- [x] Unix socket RPC
- [x] Browser extension (Chrome)
- [x] Full TUI with preview engine
- [ ] Scheduler engine (Time & OnComplete implemented)
- [x] Firefox extension
- [ ] Packaged releases (AUR, .deb, Flatpak)
- [ ] mpv IPC media position trigger
- [ ] Remote daemon access (SSH tunnel)

---

*JADMan is not affiliated with IDM or Tonec Inc.*
