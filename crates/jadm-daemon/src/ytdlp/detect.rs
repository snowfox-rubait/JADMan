use anyhow::Result;

pub async fn detect_engine(url: &str, mime_type: Option<&str>) -> Result<String> {
    let url_lower = url.to_lowercase();
    let mime = mime_type.unwrap_or("").to_lowercase();

    // 1. MIME type takes priority over URL heuristics for direct binary types.
    //    The browser already sniffed the real Content-Type — trust it.
    if !mime.is_empty() {
        // Streaming manifests → yt-dlp
        if mime.contains("mpegurl") || mime.contains("dash+xml") || mime.contains("x-mpegurl") {
            return Ok("ytdlp".to_string());
        }
        // Generic binary/octet or known archive MIME → aria2c (fast direct download)
        if mime == "application/octet-stream"
            || mime.starts_with("application/zip")
            || mime.starts_with("application/x-rar")
            || mime.starts_with("application/x-7z")
            || mime.starts_with("application/pdf")
            || mime.starts_with("image/")
        {
            return Ok("aria2c".to_string());
        }
        // Direct audio/video with a real URL (not a platform page) → aria2c is faster
        if (mime.starts_with("video/") || mime.starts_with("audio/"))
            && !url_lower.contains("youtube.com")
            && !url_lower.contains("youtu.be")
            && !url_lower.contains("vimeo.com")
            && !url_lower.contains("twitter.com")
            && !url_lower.contains("x.com")
            && !url_lower.contains("tiktok.com")
            && !url_lower.contains("instagram.com")
            && !url_lower.contains("twitch.tv")
            && !url_lower.contains("reddit.com")
        {
            return Ok("aria2c".to_string());
        }
        // text/html or application/xhtml → page URL, try yt-dlp
        if mime.contains("text/html") || mime.contains("xhtml") {
            return Ok("ytdlp".to_string());
        }
    }

    // 2. Direct binary file extensions should ALWAYS go to aria2c
    //    aria2c is faster and more reliable for raw files than yt-dlp's generic extractor.
    let direct_exts = [
        ".zip", ".rar", ".7z", ".tar.gz", ".iso", ".exe", ".msi", ".dmg", ".apk", ".bin",
        ".mp4", ".mkv", ".avi", ".mov", ".mp3", ".flac", ".wav",
        ".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp", ".svg", ".ico", ".doc", ".docx", ".pdf",
    ];
    for ext in direct_exts {
        if url_lower.split('?').next().unwrap_or("").ends_with(ext) {
            return Ok("aria2c".to_string());
        }
    }

    // 3. Known media platforms or stream manifests that strictly require yt-dlp
    let media_sites = [
        "youtube.com", "youtu.be", "vimeo.com", "twitter.com", "x.com",
        "tiktok.com", "instagram.com", "twitch.tv", "reddit.com",
        "dailymotion.com", "nicovideo.jp", "bilibili.com", "soundcloud.com",
        "bandcamp.com", "mixcloud.com", "facebook.com", "fb.watch",
    ];
    for site in media_sites {
        if url_lower.contains(site) {
            return Ok("ytdlp".to_string());
        }
    }

    if url_lower.contains(".m3u8") || url_lower.contains(".mpd") {
        return Ok("ytdlp".to_string());
    }

    // 4. If it looks like a generic webpage query (has ? or no extension), try yt-dlp
    //    Otherwise fallback to aria2c.
    let path = url_lower.split('?').next().unwrap_or(&url_lower);
    if !path.contains('.') || path.ends_with(".html") || path.ends_with(".htm") || path.ends_with(".php") {
        return Ok("ytdlp".to_string());
    }

    Ok("aria2c".to_string())
}
