use crate::ytdlp::manager::YtDlpManager;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
pub enum VideoQuality {
    Best,
    #[serde(rename = "720p")]
    P720,
    #[serde(rename = "480p")]
    P480,
}

impl VideoQuality {
    pub fn to_format_string(&self) -> &'static str {
        match self {
            VideoQuality::Best => "bv*+ba/b",
            VideoQuality::P720 => "bv*[height<=720]+ba/b",
            VideoQuality::P480 => "bv*[height<=480]+ba/b",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoContainer {
    Mp4,
    Mkv,
    Webm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    M4a,
    Aac,
    Flac,
    Wav,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadMode {
    Video {
        quality: VideoQuality,
        container: VideoContainer,
    },
    Audio {
        format: AudioFormat,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub url: String,
    pub output_dir: String,
    pub mode: DownloadMode,
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

#[derive(Clone)]
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

        let mut cmd = Command::new(self.manager.get_ytdlp_path());
        cmd.args([
            "--dump-json",
            "--flat-playlist",
            "--no-warnings",
            "--no-download",
            url,
        ]);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().await?;

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

        // Ensure output directory exists
        let output_path = std::path::Path::new(&options.output_dir);
        if !output_path.exists() {
            std::fs::create_dir_all(output_path)?;
        }

        // Build output template with proper path separator
        let output_template = output_path
            .join("%(title)s.%(ext)s")
            .to_string_lossy()
            .to_string();

        let mut args = vec![
            "--progress".to_string(),
            "--newline".to_string(),
            "--force-progress".to_string(), // Windows에서 비터미널 환경에서도 진행 상태 출력
            "-o".to_string(),
            output_template,
        ];

        // DownloadMode에 따라 인자 추가
        match &options.mode {
            DownloadMode::Video { quality, container } => {
                args.push("-f".to_string());
                args.push(quality.to_format_string().to_string());

                // 컨테이너 포맷 지정
                args.push("--merge-output-format".to_string());
                args.push(match container {
                    VideoContainer::Mp4 => "mp4",
                    VideoContainer::Mkv => "mkv",
                    VideoContainer::Webm => "webm",
                }.to_string());
            }
            DownloadMode::Audio { format } => {
                args.push("-x".to_string());
                args.push("--audio-format".to_string());
                args.push(match format {
                    AudioFormat::Mp3 => "mp3",
                    AudioFormat::M4a => "m4a",
                    AudioFormat::Aac => "aac",
                    AudioFormat::Flac => "flac",
                    AudioFormat::Wav => "wav",
                }.to_string());
            }
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

        // Emit starting status immediately
        on_progress(DownloadProgress {
            status: "starting".to_string(),
            percentage: Some(0.0),
            speed: None,
            eta: None,
            filename: None,
            total_bytes: None,
            downloaded_bytes: None,
        });

        let mut cmd = Command::new(self.manager.get_ytdlp_path());
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let progress_regex = Regex::new(
            r"\[download\]\s+(\d+\.?\d*)%\s+of\s+~?\s*([\d.]+\w+)(?:\s+at\s+([\d.]+\w+/s))?(?:\s+ETA\s+(\S+))?",
        )
        .unwrap();

        while let Ok(Some(line)) = lines.next_line().await {
            // Detect video info extraction phase
            if line.starts_with("[youtube]") || line.starts_with("[info]") || line.contains("Extracting") {
                on_progress(DownloadProgress {
                    status: "extracting".to_string(),
                    percentage: Some(0.0),
                    speed: None,
                    eta: None,
                    filename: None,
                    total_bytes: None,
                    downloaded_bytes: None,
                });
                continue;
            }
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
