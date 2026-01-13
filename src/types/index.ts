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

export interface DownloadRequest {
  url: string;
  output_dir: string;
  format: 'best' | '720p' | '480p' | 'audio';
  extract_audio: boolean;
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
  status: 'pending' | 'downloading' | 'processing' | 'completed' | 'error';
  progress: number;
  speed: string | null;
  eta: string | null;
  error: string | null;
}
