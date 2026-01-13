use crate::ytdlp::manager::YtDlpManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum DownloaderError {
    #[error("yt-dlp binary not found. Please install yt-dlp first.")]
    BinaryNotFound,
    #[error("Failed to execute yt-dlp: {0}")]
    ExecutionError(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Manager error: {0}")]
    ManagerError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadFormat {
    BestVideo,
    Video720p,
    Video480p,
    AudioOnly,
}

impl DownloadFormat {
    pub fn to_format_string(&self) -> &'static str {
        match self {
            DownloadFormat::BestVideo => "bv*+ba/b",
            DownloadFormat::Video720p => "bv*[height<=720]+ba/b",
            DownloadFormat::Video480p => "bv*[height<=480]+ba/b",
            DownloadFormat::AudioOnly => "ba/b",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub url: String,
    pub output_dir: String,
    pub format: DownloadFormat,
    pub extract_audio: bool,
    pub embed_subs: bool,
    pub playlist_items: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub uploader: Option<String>,
    pub is_playlist: bool,
    pub playlist_count: Option<usize>,
    pub entries: Option<Vec<PlaylistEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: String,
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub status: String,
    pub percentage: Option<f64>,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: Option<u64>,
}

pub struct Downloader {
    manager: YtDlpManager,
}

impl Downloader {
    pub fn new() -> Result<Self, DownloaderError> {
        let manager =
            YtDlpManager::new().map_err(|e| DownloaderError::ManagerError(e.to_string()))?;
        Ok(Self { manager })
    }

    pub async fn get_video_info(&self, url: &str) -> Result<VideoInfo, DownloaderError> {
        if !self.manager.is_ytdlp_installed() {
            return Err(DownloaderError::BinaryNotFound);
        }

        let output = Command::new(self.manager.get_ytdlp_path())
            .args([
                "--dump-json",
                "--flat-playlist",
                "--no-warnings",
                "--no-download",
                url,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(DownloaderError::ExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.is_empty() {
            return Err(DownloaderError::ExecutionError(
                "No output from yt-dlp".to_string(),
            ));
        }

        // Check if it's a playlist
        if lines.len() > 1 {
            // Multiple entries = playlist
            let mut entries = Vec::new();
            for line in &lines {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    entries.push(PlaylistEntry {
                        id: entry["id"].as_str().unwrap_or("").to_string(),
                        title: entry["title"].as_str().unwrap_or("Unknown").to_string(),
                        duration: entry["duration"].as_f64(),
                        thumbnail: entry["thumbnail"].as_str().map(|s| s.to_string()),
                    });
                }
            }

            return Ok(VideoInfo {
                id: "playlist".to_string(),
                title: "Playlist".to_string(),
                duration: None,
                thumbnail: None,
                description: None,
                uploader: None,
                is_playlist: true,
                playlist_count: Some(entries.len()),
                entries: Some(entries),
            });
        }

        // Single video
        let json: serde_json::Value = serde_json::from_str(lines[0])?;

        // Check if the single entry is a playlist reference
        if json.get("_type").and_then(|t| t.as_str()) == Some("playlist") {
            let entries: Vec<PlaylistEntry> = json
                .get("entries")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|entry| {
                            Some(PlaylistEntry {
                                id: entry["id"].as_str()?.to_string(),
                                title: entry["title"]
                                    .as_str()
                                    .unwrap_or("Unknown")
                                    .to_string(),
                                duration: entry["duration"].as_f64(),
                                thumbnail: entry["thumbnail"].as_str().map(|s| s.to_string()),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            return Ok(VideoInfo {
                id: json["id"].as_str().unwrap_or("playlist").to_string(),
                title: json["title"].as_str().unwrap_or("Playlist").to_string(),
                duration: None,
                thumbnail: json["thumbnail"].as_str().map(|s| s.to_string()),
                description: None,
                uploader: json["uploader"].as_str().map(|s| s.to_string()),
                is_playlist: true,
                playlist_count: Some(entries.len()),
                entries: Some(entries),
            });
        }

        Ok(VideoInfo {
            id: json["id"].as_str().unwrap_or("").to_string(),
            title: json["title"].as_str().unwrap_or("Unknown").to_string(),
            duration: json["duration"].as_f64(),
            thumbnail: json["thumbnail"].as_str().map(|s| s.to_string()),
            description: json["description"].as_str().map(|s| s.to_string()),
            uploader: json["uploader"].as_str().map(|s| s.to_string()),
            is_playlist: false,
            playlist_count: None,
            entries: None,
        })
    }

    pub async fn download<F>(
        &self,
        options: &DownloadOptions,
        on_progress: F,
    ) -> Result<String, DownloaderError>
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        if !self.manager.is_ytdlp_installed() {
            return Err(DownloaderError::BinaryNotFound);
        }

        let mut args = vec![
            "--progress".to_string(),
            "--newline".to_string(),
            "-f".to_string(),
            options.format.to_format_string().to_string(),
            "-o".to_string(),
            format!("{}/%(title)s.%(ext)s", options.output_dir),
        ];

        if options.extract_audio {
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
        }

        if options.embed_subs {
            args.push("--write-subs".to_string());
            args.push("--embed-subs".to_string());
        }

        if let Some(items) = &options.playlist_items {
            let items_str = items
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            args.push("--playlist-items".to_string());
            args.push(items_str);
        }

        // Add ffmpeg location if available
        if self.manager.is_ffmpeg_installed() {
            args.push("--ffmpeg-location".to_string());
            args.push(self.manager.get_ffmpeg_path().to_string_lossy().to_string());
        }

        args.push(options.url.clone());

        let mut child = Command::new(self.manager.get_ytdlp_path())
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let progress_regex = Regex::new(
            r"\[download\]\s+(\d+\.?\d*)%\s+of\s+~?\s*([\d.]+\w+)(?:\s+at\s+([\d.]+\w+/s))?(?:\s+ETA\s+(\S+))?",
        )
        .unwrap();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(caps) = progress_regex.captures(&line) {
                let percentage = caps.get(1).and_then(|m| m.as_str().parse::<f64>().ok());
                let speed = caps.get(3).map(|m| m.as_str().to_string());
                let eta = caps.get(4).map(|m| m.as_str().to_string());

                on_progress(DownloadProgress {
                    status: "downloading".to_string(),
                    percentage,
                    speed,
                    eta,
                    filename: None,
                    total_bytes: None,
                    downloaded_bytes: None,
                });
            } else if line.contains("[download] Destination:") {
                let filename = line.replace("[download] Destination:", "").trim().to_string();
                on_progress(DownloadProgress {
                    status: "starting".to_string(),
                    percentage: Some(0.0),
                    speed: None,
                    eta: None,
                    filename: Some(filename),
                    total_bytes: None,
                    downloaded_bytes: None,
                });
            } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") {
                on_progress(DownloadProgress {
                    status: "processing".to_string(),
                    percentage: Some(100.0),
                    speed: None,
                    eta: None,
                    filename: None,
                    total_bytes: None,
                    downloaded_bytes: None,
                });
            }
        }

        let status = child.wait().await?;

        if status.success() {
            on_progress(DownloadProgress {
                status: "completed".to_string(),
                percentage: Some(100.0),
                speed: None,
                eta: None,
                filename: None,
                total_bytes: None,
                downloaded_bytes: None,
            });
            Ok(options.output_dir.clone())
        } else {
            Err(DownloaderError::DownloadFailed(
                "Download process failed".to_string(),
            ))
        }
    }

    pub fn get_manager(&self) -> &YtDlpManager {
        &self.manager
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create Downloader")
    }
}
