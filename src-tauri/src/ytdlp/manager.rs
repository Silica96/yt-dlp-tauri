use directories::ProjectDirs;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Failed to get app data directory")]
    NoAppDataDir,
    #[error("yt-dlp binary not found")]
    BinaryNotFound,
    #[error("Failed to execute yt-dlp: {0}")]
    ExecutionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct YtDlpManager {
    bin_dir: PathBuf,
}

impl YtDlpManager {
    pub fn new() -> Result<Self, ManagerError> {
        let project_dirs = ProjectDirs::from("com", "gyuseok", "yt-dlp-gui")
            .ok_or(ManagerError::NoAppDataDir)?;

        let bin_dir = project_dirs.data_dir().join("bin");

        // Create bin directory if it doesn't exist
        std::fs::create_dir_all(&bin_dir)?;

        Ok(Self { bin_dir })
    }

    pub fn get_bin_dir(&self) -> &PathBuf {
        &self.bin_dir
    }

    pub fn get_ytdlp_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.bin_dir.join("yt-dlp.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.bin_dir.join("yt-dlp")
        }
    }

    pub fn get_ffmpeg_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.bin_dir.join("ffmpeg.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.bin_dir.join("ffmpeg")
        }
    }

    pub fn is_ytdlp_installed(&self) -> bool {
        self.get_ytdlp_path().exists()
    }

    pub fn is_ffmpeg_installed(&self) -> bool {
        self.get_ffmpeg_path().exists()
    }

    pub fn get_ytdlp_version(&self) -> Result<String, ManagerError> {
        if !self.is_ytdlp_installed() {
            return Err(ManagerError::BinaryNotFound);
        }

        let mut cmd = Command::new(self.get_ytdlp_path());
        cmd.arg("--version");

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(ManagerError::ExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub fn get_download_url() -> (&'static str, &'static str) {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
                "yt-dlp",
            )
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
                "yt-dlp",
            )
        }
        #[cfg(target_os = "windows")]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
                "yt-dlp.exe",
            )
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux",
                "yt-dlp",
            )
        }
        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            target_os = "windows",
            all(target_os = "linux", target_arch = "x86_64")
        )))]
        {
            (
                "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp",
                "yt-dlp",
            )
        }
    }

    pub fn get_ffmpeg_download_url() -> Option<&'static str> {
        #[cfg(target_os = "macos")]
        {
            Some("https://evermeet.cx/ffmpeg/getrelease/zip")
        }
        #[cfg(target_os = "windows")]
        {
            Some("https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip")
        }
        #[cfg(target_os = "linux")]
        {
            // On Linux, ffmpeg should be installed via package manager
            None
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            None
        }
    }

    pub fn get_app_data_dir() -> Result<PathBuf, ManagerError> {
        let project_dirs = ProjectDirs::from("com", "gyuseok", "yt-dlp-gui")
            .ok_or(ManagerError::NoAppDataDir)?;
        Ok(project_dirs.data_dir().to_path_buf())
    }

    pub fn get_default_download_dir() -> PathBuf {
        directories::UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

impl Default for YtDlpManager {
    fn default() -> Self {
        Self::new().expect("Failed to create YtDlpManager")
    }
}
