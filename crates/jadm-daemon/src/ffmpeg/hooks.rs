use std::path::Path;
use anyhow::Result;

#[allow(dead_code)]
pub async fn extract_thumbnail(_input: &Path, _output: &Path) -> Result<()> {
    // tokio::process::Command::new("ffmpeg")
    //     .args(["-i", input.to_str().unwrap(), "-ss", "00:00:01", "-vframes", "1", output.to_str().unwrap()])
    //     .output().await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn remux_to_mp4(_input: &Path, _output: &Path) -> Result<()> {
    // tokio::process::Command::new("ffmpeg")
    //     .args(["-i", input.to_str().unwrap(), "-codec", "copy", output.to_str().unwrap()])
    //     .output().await?;
    Ok(())
}
