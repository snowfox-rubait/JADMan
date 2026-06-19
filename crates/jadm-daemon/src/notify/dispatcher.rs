use anyhow::Result;
use notify_rust::Notification;

#[allow(dead_code)]
pub struct NotificationDispatcher {
    enabled: bool,
}

impl NotificationDispatcher {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    #[allow(dead_code)]
    pub fn send_completion(&self, filename: &str) -> Result<()> {
        if !self.enabled { return Ok(()); }
        Notification::new()
            .summary("JADMan: Download Complete")
            .body(&format!("Finished downloading {}", filename))
            .icon("download")
            .show()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_error(&self, filename: &str, error: &str) -> Result<()> {
        if !self.enabled { return Ok(()); }
        Notification::new()
            .summary("JADMan: Download Failed")
            .body(&format!("Error downloading {}: {}", filename, error))
            .icon("error")
            .show()?;
        Ok(())
    }
}
