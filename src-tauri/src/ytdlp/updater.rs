use crate::ytdlp::manager::YtDlpManager;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdaterError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse version info")]
    ParseError,
    #[error("Manager error: {0}")]
    ManagerError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub tag_name: String,
    pub published_at: String,
    pub html_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatus {
    pub installed: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgressEvent {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percentage: Option<f64>,
}

pub struct Updater {
    client: Client,
    manager: YtDlpManager,
}

impl Updater {
    pub fn new() -> Result<Self, UpdaterError> {
        let manager = YtDlpManager::new().map_err(|e| UpdaterError::ManagerError(e.to_string()))?;
        Ok(Self {
            client: Client::new(),
            manager,
        })
    }

    pub async fn get_latest_version(&self) -> Result<VersionInfo, UpdaterError> {
        let response = self
            .client
            .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
            .header("User-Agent", "yt-dlp-gui")
            .send()
            .await?;

        let release: serde_json::Value = response.json().await?;

        Ok(VersionInfo {
            tag_name: release["tag_name"]
                .as_str()
                .ok_or(UpdaterError::ParseError)?
                .to_string(),
            published_at: release["published_at"]
                .as_str()
                .ok_or(UpdaterError::ParseError)?
                .to_string(),
            html_url: release["html_url"]
                .as_str()
                .ok_or(UpdaterError::ParseError)?
                .to_string(),
        })
    }

    pub async fn check_update_status(&self) -> Result<UpdateStatus, UpdaterError> {
        let installed = self.manager.is_ytdlp_installed();

        let current_version = if installed {
            self.manager.get_ytdlp_version().ok()
        } else {
            None
        };

        let latest_info = self.get_latest_version().await.ok();
        let latest_version = latest_info.map(|v| v.tag_name);

        let update_available = match (&current_version, &latest_version) {
            (Some(current), Some(latest)) => current != latest,
            (None, Some(_)) => true,
            _ => false,
        };

        Ok(UpdateStatus {
            installed,
            current_version,
            latest_version,
            update_available,
        })
    }

    pub async fn download_ytdlp<F>(&self, on_progress: F) -> Result<PathBuf, UpdaterError>
    where
        F: Fn(DownloadProgressEvent),
    {
        let (url, filename) = YtDlpManager::get_download_url();
        let dest_path = self.manager.get_bin_dir().join(filename);

        // Create temp file
        let temp_path = dest_path.with_extension("tmp");

        let response = self
            .client
            .get(url)
            .header("User-Agent", "yt-dlp-gui")
            .send()
            .await?;

        let total_size = response.content_length();
        let mut downloaded: u64 = 0;
        let mut file = std::fs::File::create(&temp_path)?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            on_progress(DownloadProgressEvent {
                downloaded,
                total: total_size,
                percentage: total_size.map(|t| (downloaded as f64 / t as f64) * 100.0),
            });
        }

        // Flush and close file
        drop(file);

        // Move temp file to final location
        std::fs::rename(&temp_path, &dest_path)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest_path, perms)?;
        }

        Ok(dest_path)
    }

    pub fn get_manager(&self) -> &YtDlpManager {
        &self.manager
    }
}

impl Default for Updater {
    fn default() -> Self {
        Self::new().expect("Failed to create Updater")
    }
}
