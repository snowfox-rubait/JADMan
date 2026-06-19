# JADM Daemon Features

## Status: Core Engine

### Engine & Routing
- [x] **Automatic Routing:** Detects if a URL should use `aria2c` (standard) or `yt-dlp` (media/streams).
- [x] **Persistence:** SQLite database (jadm.db) for session-resilient queue management.
- [x] **Format Detection:** `/formats` endpoint that uses yt-dlp to query available video/audio qualities.

### Connectivity
- [x] **Unix RPC:** High-performance socket-based communication for the TUI.
- [x] **HTTP REST API:** Axum-based server for extensions, WebUI, and external scripts.
- [x] **CORS Support:** Permissive CORS for seamless integration with browser tools.

### Automation
- [x] **Clipboard Monitor:** Automatically detects URLs in the clipboard and adds them if configured.
- [x] **Scheduler:** Rule-based engine to start/stop downloads at specific times.
- [x] **System Notifications:** Integration with `notify-rust` for desktop alerts.

### Planned Features
- [ ] **Category Auto-Sorting:** Automatically move files based on extension (e.g., .mp4 to Videos).
- [ ] **Bandwidth Limiting:** Global and per-download speed caps.
- [ ] **Proxy Support:** Integration for SOCKS/HTTP proxies.
