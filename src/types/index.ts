export interface AppStatus {
  ytdlp_installed: boolean;
  ffmpeg_installed: boolean;
  ytdlp_version: string | null;
  default_download_dir: string;
}

export interface UpdateStatus {
  installed: boolean;
  current_version: string | null;
  latest_version: string | null;
  update_available: boolean;
}

export interface VideoInfo {
  id: string;
  title: string;
  duration: number | null;
  thumbnail: string | null;
  description: string | null;
  uploader: string | null;
  is_playlist: boolean;
  playlist_count: number | null;
  entries: PlaylistEntry[] | null;
}

export interface PlaylistEntry {
  id: string;
  title: string;
  duration: number | null;
  thumbnail: string | null;
}

export type VideoQuality = 'best' | '720p' | '480p';
export type VideoContainer = 'mp4' | 'mkv' | 'webm';
export type AudioFormat = 'mp3' | 'm4a' | 'aac' | 'flac' | 'wav';

export interface DownloadRequest {
  url: string;
  output_dir: string;
  // 비디오 옵션 (둘 다 있거나 둘 다 없음)
  video_quality?: VideoQuality;
  video_container?: VideoContainer;
  // 오디오 옵션
  audio_format?: AudioFormat;
  // 기존 필드
  embed_subs: boolean;
  playlist_items: number[] | null;
}

export interface DownloadProgress {
  id: string;
  status: string;
  percentage: number | null;
  speed: string | null;
  eta: string | null;
  filename: string | null;
}

export interface YtDlpDownloadProgress {
  downloaded: number;
  total: number | null;
  percentage: number | null;
}

export interface DownloadItem {
  id: string;
  url: string;
  title: string;
  status: 'pending' | 'starting' | 'extracting' | 'downloading' | 'processing' | 'completed' | 'error';
  progress: number;
  speed: string | null;
  eta: string | null;
  error: string | null;
}
