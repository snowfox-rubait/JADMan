use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{Style, ThemeSet};
use syntect::util::LinesWithEndings;
use ratatui::text::{Line, Span, Text};
use ratatui::style::{Style as RatatuiStyle, Color};
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct TextPreviewer {
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl TextPreviewer {
    pub fn new() -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    pub fn preview(&self, path: &Path) -> Text<'static> {
        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Text::raw(format!("Error opening file: {}", e)),
        };

        let mut buffer = vec![0u8; 65536];
        let bytes_read = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => return Text::raw(format!("Error reading file: {}", e)),
        };
        buffer.truncate(bytes_read);

        if buffer.iter().any(|&b| b == 0) {
            return Text::raw("  [Binary file — preview not available]");
        }

        let content = String::from_utf8_lossy(&buffer);

        // Take first 100 lines for preview
        let preview_content: String = content.lines().take(100).collect::<Vec<_>>().join("\n");

        let syntax = self.ps.find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());
        
        let mut h = HighlightLines::new(syntax, &self.ts.themes["base16-ocean.dark"]);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(&preview_content) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.ps).unwrap();
            let mut spans = Vec::new();
            for (style, text) in ranges {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                spans.push(Span::styled(text.to_string(), RatatuiStyle::default().fg(fg)));
            }
            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }
}
