import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { VideoInfo, DownloadRequest, DownloadProgress, DownloadItem } from '../types';

export const useDownloadStore = defineStore('download', () => {
  const downloads = ref<DownloadItem[]>([]);
  const currentVideoInfo = ref<VideoInfo | null>(null);
  const isAnalyzing = ref(false);
  const analyzeError = ref<string | null>(null);
  const downloadDir = ref('');

  let progressUnlisten: UnlistenFn | null = null;

  const activeDownloads = computed(() =>
    downloads.value.filter(d => ['starting', 'extracting', 'downloading', 'processing'].includes(d.status))
  );

  const completedDownloads = computed(() =>
    downloads.value.filter(d => d.status === 'completed')
  );

  async function analyzeUrl(url: string) {
    isAnalyzing.value = true;
    analyzeError.value = null;
    currentVideoInfo.value = null;

    try {
      currentVideoInfo.value = await invoke<VideoInfo>('get_video_info', { url });
    } catch (e) {
      analyzeError.value = String(e);
    } finally {
      isAnalyzing.value = false;
    }
  }

  async function startDownload(request: DownloadRequest, title: string) {
    // Setup progress listener if not already set
    if (!progressUnlisten) {
      progressUnlisten = await listen<DownloadProgress>('download-progress', (event) => {
        const { id, status, percentage, speed, eta, filename } = event.payload;
        const download = downloads.value.find(d => d.id === id);
        if (download) {
          download.status = status as DownloadItem['status'];
          download.progress = percentage ?? 0;
          download.speed = speed;
          download.eta = eta;
          if (filename) {
            download.title = filename.split('/').pop() ?? title;
          }
        }
      });
    }

    try {
      const id = await invoke<string>('start_download', { request });

      downloads.value.unshift({
        id,
        url: request.url,
        title,
        status: 'pending',
        progress: 0,
        speed: null,
        eta: null,
        error: null,
      });

      return id;
    } catch (e) {
      const errorItem: DownloadItem = {
        id: crypto.randomUUID(),
        url: request.url,
        title,
        status: 'error',
        progress: 0,
        speed: null,
        eta: null,
        error: String(e),
      };
      downloads.value.unshift(errorItem);
      throw e;
    }
  }

  function clearCompleted() {
    downloads.value = downloads.value.filter(d => d.status !== 'completed');
  }

  function removeDownload(id: string) {
    downloads.value = downloads.value.filter(d => d.id !== id);
  }

  function setDownloadDir(dir: string) {
    downloadDir.value = dir;
  }

  function cleanup() {
    if (progressUnlisten) {
      progressUnlisten();
      progressUnlisten = null;
    }
  }

  return {
    downloads,
    currentVideoInfo,
    isAnalyzing,
    analyzeError,
    downloadDir,
    activeDownloads,
    completedDownloads,
    analyzeUrl,
    startDownload,
    clearCompleted,
    removeDownload,
    setDownloadDir,
    cleanup,
  };
});
