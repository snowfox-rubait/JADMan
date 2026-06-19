# JADM Extension Features

## Status: Active Development

### Interception & Detection
- [x] **Standard Interception:** Catches common file extensions (.zip, .rar, .iso, etc.) from standard browser downloads.
- [x] **Network Sniffer:** Monitors MIME types in the background to detect hidden media streams (MP4, WebM, M3U8, DASH).
- [x] **YouTube Shorts Support:** Specifically detects and enables downloads for YouTube Shorts player.
- [x] **Vimeo Support:** Basic detection for Vimeo video pages.

### User Interface
- [x] **Floating Media Bar:** Draggable red bar injected into video players for quick "one-click" access.
- [x] **Quality Selector Popup:** A dedicated popup window that fetches available formats from the daemon before starting.
- [x] **Batch Grabber:** A powerful popup triggered by right-clicking a text selection to batch-download multiple links.

### Planned Features
- [ ] **Advanced Grabber Filters:** Filter by file size or date if available in headers.
- [ ] **Cookie Passing:** Automatically send browser cookies to aria2c/yt-dlp for authenticated downloads (e.g., premium sites).
- [ ] **Context Menu "Download Link":** Right-click a single link to send directly to JADM.
