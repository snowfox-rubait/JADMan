use std::path::Path;
use ratatui::text::Text;
use crate::preview::text::TextPreviewer;

pub enum PreviewData<'a> {
    Text(Text<'a>),
    Image, // Placeholder for Phase 2/3
    Video, // Placeholder for Phase 2/3
    None,
}

pub struct PreviewEngine {
    text_previewer: TextPreviewer,
}

impl PreviewEngine {
    pub fn new() -> Self {
        Self {
            text_previewer: TextPreviewer::new(),
        }
    }

    pub fn get_preview(&self, path_str: &str) -> PreviewData<'_> {
        let path = Path::new(path_str);
        if !path.exists() {
            return PreviewData::None;
        }

        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" | "rs" | "toml" | "js" | "py" | "sh" | "json" => {
                PreviewData::Text(self.text_previewer.preview(path))
            }
            "jpg" | "png" | "gif" | "webp" => PreviewData::Image,
            "mp4" | "mkv" | "webm" => PreviewData::Video,
            _ => {
                // Try text preview for unknown extensions as fallback
                PreviewData::Text(self.text_previewer.preview(path))
            }
        }
    }
}
