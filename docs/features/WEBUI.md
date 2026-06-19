# JADM WebUI Features

## Status: Functional Debugger / Alternative UI

### Core Features
- [x] **Interface Parity:** Layout matches the TUI for a consistent user experience.
- [x] **Interactive Dashboard:** Modern dark-themed dashboard accessible at `http://localhost:6246`.
- [x] **Live Polling:** Automatic 1-second refresh of download statuses.

### Remote Control
- [x] **Task Actions:** Resume, Pause, Stop, and Delete buttons fully wired to the daemon.
- [x] **Detail View:** Inspection of URL, filename, and specific download paths.

### Planned Features
- [ ] **Full Preview Support:** Display image thumbnails and video previews (using the ffmpeg hooks).
- [ ] **Drag-and-Drop:** Drop files/links onto the WebUI to add them to the queue.
- [ ] **Configuration UI:** Web-based interface for managing daemon settings and categories.
