use ratatui::style::{Color, Modifier, Style};

// ── Base palette ──────────────────────────────────────────────────
pub const BG:           Color = Color::Rgb(18, 18, 24);
pub const BG_ALT:       Color = Color::Rgb(24, 24, 32);
pub const SURFACE:      Color = Color::Rgb(30, 30, 42);
pub const BORDER:       Color = Color::Rgb(60, 60, 80);
pub const BORDER_ACTIVE:Color = Color::Rgb(100, 200, 255);
pub const TEXT:         Color = Color::Rgb(200, 200, 215);
pub const TEXT_DIM:     Color = Color::Rgb(120, 120, 140);
pub const TEXT_BRIGHT:  Color = Color::Rgb(240, 240, 255);

// ── Accent colors ─────────────────────────────────────────────────
pub const ACCENT:       Color = Color::Rgb(100, 200, 255);  // cyan
pub const GREEN:        Color = Color::Rgb(80, 220, 120);
pub const RED:          Color = Color::Rgb(255, 90, 90);
pub const YELLOW:       Color = Color::Rgb(255, 210, 70);
pub const ORANGE:       Color = Color::Rgb(255, 160, 50);

// ── Selection ─────────────────────────────────────────────────────
pub const SELECTED_BG:  Color = Color::Rgb(40, 40, 70);
pub const SELECTED_FG:  Color = Color::Rgb(240, 240, 255);

// ── Status-specific colors ────────────────────────────────────────
pub const STATUS_DOWNLOADING: Color = Color::Rgb(100, 200, 255);
pub const STATUS_DONE:        Color = Color::Rgb(80, 220, 120);
pub const STATUS_FAILED:      Color = Color::Rgb(255, 90, 90);
pub const STATUS_PAUSED:      Color = Color::Rgb(255, 210, 70);
pub const STATUS_QUEUED:      Color = Color::Rgb(120, 120, 140);
pub const STATUS_CANCELLED:   Color = Color::Rgb(180, 100, 100);

// ── Convenience styles ────────────────────────────────────────────
pub fn base_style() -> Style {
    Style::default().fg(TEXT).bg(BG)
}

pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM).bg(BG)
}

pub fn header_style() -> Style {
    Style::default().fg(ACCENT).bg(SURFACE).add_modifier(Modifier::BOLD)
}

pub fn border_style(active: bool) -> Style {
    if active {
        Style::default().fg(BORDER_ACTIVE).bg(BG)
    } else {
        Style::default().fg(BORDER).bg(BG)
    }
}

pub fn selected_style() -> Style {
    Style::default().fg(SELECTED_FG).bg(SELECTED_BG)
}

pub fn row_style(index: usize) -> Style {
    if index % 2 == 0 {
        Style::default().fg(TEXT).bg(BG)
    } else {
        Style::default().fg(TEXT).bg(BG_ALT)
    }
}

use jadm_common::types::DownloadStatus;

pub fn status_color(status: &DownloadStatus) -> Color {
    match status {
        DownloadStatus::Downloading => STATUS_DOWNLOADING,
        DownloadStatus::Done        => STATUS_DONE,
        DownloadStatus::Failed      => STATUS_FAILED,
        DownloadStatus::Paused      => STATUS_PAUSED,
        DownloadStatus::Queued      => STATUS_QUEUED,
        DownloadStatus::Cancelled   => STATUS_CANCELLED,
    }
}

pub fn status_icon(status: &DownloadStatus) -> &'static str {
    match status {
        DownloadStatus::Downloading => "↓ ",
        DownloadStatus::Done        => "✓ ",
        DownloadStatus::Failed      => "✗ ",
        DownloadStatus::Paused      => "⏸ ",
        DownloadStatus::Queued      => "◦ ",
        DownloadStatus::Cancelled   => "⊘ ",
    }
}

/// Progress bar color gradient: red → yellow → green
pub fn progress_color(percent: u8) -> Color {
    match percent {
        0..=25   => Color::Rgb(255, 70, 70),
        26..=50  => Color::Rgb(255, 160, 50),
        51..=75  => Color::Rgb(255, 210, 70),
        76..=99  => Color::Rgb(140, 220, 100),
        _        => Color::Rgb(80, 220, 120),
    }
}
