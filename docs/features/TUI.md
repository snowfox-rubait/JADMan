# JADM TUI Features

## Status: Stable Prototype

### Interface Structure
- [x] **Three-Panel Design:** Navigation (Categories), Main List (Queue), and Inspection (Detail/Preview).
- [x] **Vim-Inspired Navigation:** Support for `h`, `j`, `k`, `l` and common TUI shortcuts.
- [x] **Dynamic Columns:** Real-time display of Size, Rate, ETA, and Status.
- [x] **Visual Progress:** Custom Pacman-style progress bars (`[#######c   ]`).

### Functionality
- [x] **Preview Engine:** Syntax-highlighted text previews for downloaded files using `syntect`.
- [x] **Queue Management:** Bounds-checked selection that prevents crashes when the queue changes.
- [x] **Real-Time Polling:** Synchronized with the daemon via Unix RPC.

### Planned Features
- [ ] **Multi-Select:** Select multiple items in the list for batch actions (Pause All, Delete All).
- [ ] **Config Editor:** Edit JADM configuration files directly inside the TUI.
- [ ] **Interactive Search:** `/` to filter the download list by name.
