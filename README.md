# JADMan — Just Another Download Manager

> The download manager Linux deserves. A full IDM replacement — and then some.

JADMan is a keyboard-driven, terminal-native download manager for Linux, built in Rust.
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

## Installation

> JADMan is in active development. Pre-built binaries are coming soon.
> For now, build from source.

### Prerequisites

```bash
# Required
aria2c
yt-dlp
ffmpeg

# Install via your package manager, e.g.:
sudo pacman -S aria2 yt-dlp ffmpeg        # Arch
sudo apt install aria2 yt-dlp ffmpeg      # Debian/Ubuntu
```

### Build from Source

```bash
git clone https://github.com/snowfox-rubait/JADMan
cd JADMan
cargo build --release

# Binaries will be at:
# target/release/jadm-daemon   (daemon)
# target/release/jadm-tui      (TUI)
```

### Browser Extension

1. Open Chrome → `chrome://extensions`
2. Enable **Developer mode**
3. Click **Load unpacked** → select the `extension/chrome/` folder

Firefox support: located in `extension/firefox/`.

---

## Usage

```bash
# Start the daemon
jadm-daemon &

# Open the TUI
jadm-tui

# Or add a download directly from the CLI (via RPC)
# jadm-tui add https://example.com/file.zip
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
