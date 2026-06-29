use jadm_common::types::{DownloadStatus, DownloadView};
use crate::preview::engine::PreviewEngine;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Panel {
    Categories,
    DownloadList,
    Detail,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CategoryFilter {
    All,
    Downloading,
    Finished,
    Paused,
    Failed,
    Queued,
}

impl CategoryFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Downloads",
            Self::Downloading => "Downloading",
            Self::Finished => "Finished",
            Self::Paused => "Paused",
            Self::Failed => "Failed",
            Self::Queued => "Queued",
        }
    }

    pub fn matches(&self, status: &DownloadStatus) -> bool {
        match self {
            Self::All => true,
            Self::Downloading => matches!(status, DownloadStatus::Downloading),
            Self::Finished => matches!(status, DownloadStatus::Done),
            Self::Paused => matches!(status, DownloadStatus::Paused),
            Self::Failed => matches!(status, DownloadStatus::Failed | DownloadStatus::Cancelled),
            Self::Queued => matches!(status, DownloadStatus::Queued),
        }
    }

    pub const ALL_FILTERS: &'static [CategoryFilter] = &[
        CategoryFilter::All,
        CategoryFilter::Downloading,
        CategoryFilter::Finished,
        CategoryFilter::Paused,
        CategoryFilter::Failed,
        CategoryFilter::Queued,
    ];
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortBy {
    Name,
    Size,
    DateAdded,
    TimeLeft,
    Rate,
    Status,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    AddUrl,
    ConfirmDelete,
    Help,
    CookiePassword,
}

#[allow(dead_code)]
pub struct App {
    pub downloads: Vec<DownloadView>,
    pub selected_index: usize,
    pub focused_panel: Panel,
    pub running: bool,
    pub preview_engine: PreviewEngine,
    pub category_filter: CategoryFilter,
    pub category_index: usize,
    pub sort_by: SortBy,
    pub sort_reverse: bool,
    pub selected_ids: HashSet<Uuid>,
    pub pending_key: Option<char>,
    pub scroll_offset: u16,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub show_help: bool,
    pub connected: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            downloads: Vec::new(),
            selected_index: 0,
            focused_panel: Panel::DownloadList,
            running: true,
            preview_engine: PreviewEngine::new(),
            category_filter: CategoryFilter::All,
            category_index: 0,
            sort_by: SortBy::DateAdded,
            sort_reverse: false,
            selected_ids: HashSet::new(),
            pending_key: None,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            show_help: false,
            connected: false,
            status_message: None,
        }
    }

    /// Returns downloads filtered by the current category filter.
    pub fn filtered_downloads(&self) -> Vec<&DownloadView> {
        self.downloads
            .iter()
            .filter(|dv| self.category_filter.matches(&dv.download.status))
            .collect()
    }

    pub fn next_download(&mut self) {
        let count = self.filtered_downloads().len();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    pub fn prev_download(&mut self) {
        let count = self.filtered_downloads().len();
        if count > 0 {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = count - 1;
            }
        }
    }

    pub fn check_bounds(&mut self) {
        let count = self.filtered_downloads().len();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count - 1;
        }
    }

    pub fn next_category(&mut self) {
        let filters = CategoryFilter::ALL_FILTERS;
        if self.category_index + 1 < filters.len() {
            self.category_index += 1;
        } else {
            self.category_index = 0;
        }
        self.category_filter = filters[self.category_index].clone();
        self.selected_index = 0;
    }

    pub fn prev_category(&mut self) {
        let filters = CategoryFilter::ALL_FILTERS;
        if self.category_index > 0 {
            self.category_index -= 1;
        } else {
            self.category_index = filters.len() - 1;
        }
        self.category_filter = filters[self.category_index].clone();
        self.selected_index = 0;
    }

    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Categories => Panel::DownloadList,
            Panel::DownloadList => Panel::Detail,
            Panel::Detail => Panel::Categories,
        };
    }

    pub fn prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Categories => Panel::Detail,
            Panel::DownloadList => Panel::Categories,
            Panel::Detail => Panel::DownloadList,
        };
    }

    /// Count downloads matching a given status.
    pub fn count_by_status(&self, status: &DownloadStatus) -> usize {
        self.downloads
            .iter()
            .filter(|dv| dv.download.status == *status)
            .count()
    }

    pub fn active_count(&self) -> usize {
        self.count_by_status(&DownloadStatus::Downloading)
    }

    /// Get the currently selected download (respecting category filter).
    pub fn selected_download(&self) -> Option<&DownloadView> {
        let filtered = self.filtered_downloads();
        filtered.get(self.selected_index).copied()
    }
}
