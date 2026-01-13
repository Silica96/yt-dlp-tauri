mod ytdlp;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;

use ytdlp::downloader::{AudioFormat, DownloadMode, DownloadOptions, Downloader, VideoContainer, VideoInfo, VideoQuality};
use ytdlp::manager::YtDlpManager;
use ytdlp::updater::{UpdateStatus, Updater};

// App state
pub struct AppState {
    downloader: Arc<Mutex<Option<Downloader>>>,
    updater: Arc<Mutex<Option<Updater>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            downloader: Arc::new(Mutex::new(None)),
            updater: Arc::new(Mutex::new(None)),
        }
    }
}

// Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AppStatus {
    pub ytdlp_installed: bool,
    pub ffmpeg_installed: bool,
    pub ytdlp_version: Option<String>,
    pub default_download_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub status: String,
    pub percentage: Option<f64>,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct YtDlpDownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percentage: Option<f64>,
}

// Commands
#[tauri::command]
async fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, String> {
    let manager = YtDlpManager::new().map_err(|e| e.to_string())?;

    let ytdlp_installed = manager.is_ytdlp_installed();
    let ffmpeg_installed = manager.is_ffmpeg_installed();

    // Initialize downloader and updater in background (don't block startup)
    if ytdlp_installed {
        let state_clone = state.downloader.clone();
        tokio::spawn(async move {
            let mut downloader_guard = state_clone.lock().await;
            if downloader_guard.is_none() {
                if let Ok(downloader) = Downloader::new() {
                    *downloader_guard = Some(downloader);
                }
            }
        });
    }

    {
        let state_clone = state.updater.clone();
        tokio::spawn(async move {
            let mut updater_guard = state_clone.lock().await;
            if updater_guard.is_none() {
                if let Ok(updater) = Updater::new() {
                    *updater_guard = Some(updater);
                }
            }
        });
    }

    Ok(AppStatus {
        ytdlp_installed,
        ffmpeg_installed,
        ytdlp_version: None, // Load version lazily to speed up startup
        default_download_dir: YtDlpManager::get_default_download_dir()
            .to_string_lossy()
            .to_string(),
    })
}

#[tauri::command]
async fn get_ytdlp_version() -> Result<Option<String>, String> {
    let manager = YtDlpManager::new().map_err(|e| e.to_string())?;
    if manager.is_ytdlp_installed() {
        Ok(manager.get_ytdlp_version().ok())
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn check_update(state: State<'_, AppState>) -> Result<UpdateStatus, String> {
    let updater_guard = state.updater.lock().await;
    let updater = updater_guard.as_ref().ok_or("Updater not initialized")?;

    updater.check_update_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_ytdlp(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let updater_guard = state.updater.lock().await;
    let updater = updater_guard.as_ref().ok_or("Updater not initialized")?;

    let app_clone = app.clone();
    let path = updater
        .download_ytdlp(move |progress| {
            let _ = app_clone.emit("ytdlp-download-progress", YtDlpDownloadProgress {
                downloaded: progress.downloaded,
                total: progress.total,
                percentage: progress.percentage,
            });
        })
        .await
        .map_err(|e| e.to_string())?;

    // Reinitialize downloader after installation
    drop(updater_guard);
    let mut downloader_guard = state.downloader.lock().await;
    if let Ok(downloader) = Downloader::new() {
        *downloader_guard = Some(downloader);
    }

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_video_info(url: String, state: State<'_, AppState>) -> Result<VideoInfo, String> {
    let downloader_guard = state.downloader.lock().await;
    let downloader = downloader_guard
        .as_ref()
        .ok_or("Downloader not initialized. Please install yt-dlp first.")?;

    downloader
        .get_video_info(&url)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct StartDownloadRequest {
    pub url: String,
    pub output_dir: String,
    // 비디오 옵션
    pub video_quality: Option<String>,
    pub video_container: Option<String>,
    // 오디오 옵션
    pub audio_format: Option<String>,
    // 기존
    pub embed_subs: bool,
    pub playlist_items: Option<Vec<usize>>,
}

#[tauri::command]
async fn start_download(
    app: AppHandle,
    request: StartDownloadRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let downloader_guard = state.downloader.lock().await;
    let downloader = downloader_guard
        .as_ref()
        .ok_or("Downloader not initialized. Please install yt-dlp first.")?
        .clone();

    let mode = if let Some(audio_fmt) = request.audio_format {
        // 오디오 모드
        let format = match audio_fmt.as_str() {
            "mp3" => AudioFormat::Mp3,
            "m4a" => AudioFormat::M4a,
            "aac" => AudioFormat::Aac,
            "flac" => AudioFormat::Flac,
            "wav" => AudioFormat::Wav,
            _ => AudioFormat::Mp3, // 기본값
        };
        DownloadMode::Audio { format }
    } else {
        // 비디오 모드
        let quality = match request.video_quality.as_deref() {
            Some("720p") => VideoQuality::P720,
            Some("480p") => VideoQuality::P480,
            _ => VideoQuality::Best, // 기본값
        };
        let container = match request.video_container.as_deref() {
            Some("mkv") => VideoContainer::Mkv,
            Some("webm") => VideoContainer::Webm,
            _ => VideoContainer::Mp4, // 기본값
        };
        DownloadMode::Video { quality, container }
    };

    let options = DownloadOptions {
        url: request.url,
        output_dir: request.output_dir,
        mode,
        embed_subs: request.embed_subs,
        playlist_items: request.playlist_items,
    };

    let download_id = Uuid::new_v4().to_string();
    let download_id_for_progress = download_id.clone();
    let download_id_for_error = download_id.clone();
    let app_for_progress = app.clone();
    let app_for_error = app.clone();

    // Spawn download task in background and return immediately
    tokio::spawn(async move {
        let result = downloader
            .download(&options, move |progress| {
                let _ = app_for_progress.emit(
                    "download-progress",
                    DownloadProgressEvent {
                        id: download_id_for_progress.clone(),
                        status: progress.status.clone(),
                        percentage: progress.percentage,
                        speed: progress.speed.clone(),
                        eta: progress.eta.clone(),
                        filename: progress.filename.clone(),
                    },
                );
            })
            .await;

        if let Err(e) = result {
            let _ = app_for_error.emit(
                "download-progress",
                DownloadProgressEvent {
                    id: download_id_for_error,
                    status: "error".to_string(),
                    percentage: None,
                    speed: None,
                    eta: None,
                    filename: Some(e.to_string()),
                },
            );
        }
    });

    // Release the lock before returning
    drop(downloader_guard);

    Ok(download_id)
}

#[tauri::command]
fn get_default_download_dir() -> String {
    YtDlpManager::get_default_download_dir()
        .to_string_lossy()
        .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_ytdlp_version,
            check_update,
            download_ytdlp,
            get_video_info,
            start_download,
            get_default_download_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
