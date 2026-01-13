import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppStatus, UpdateStatus, YtDlpDownloadProgress } from '../types';

export const useAppStore = defineStore('app', () => {
  const status = ref<AppStatus | null>(null);
  const updateStatus = ref<UpdateStatus | null>(null);
  const isLoading = ref(false);
  const isInstalling = ref(false);
  const installProgress = ref(0);
  const error = ref<string | null>(null);

  const isReady = computed(() => status.value?.ytdlp_installed ?? false);
  const defaultDownloadDir = computed(() => status.value?.default_download_dir ?? '');

  async function fetchStatus() {
    isLoading.value = true;
    error.value = null;
    try {
      status.value = await invoke<AppStatus>('get_app_status');
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function checkUpdate() {
    try {
      updateStatus.value = await invoke<UpdateStatus>('check_update');
    } catch (e) {
      console.error('Failed to check update:', e);
    }
  }

  async function installYtDlp() {
    isInstalling.value = true;
    installProgress.value = 0;
    error.value = null;

    const unlisten = await listen<YtDlpDownloadProgress>('ytdlp-download-progress', (event) => {
      installProgress.value = event.payload.percentage ?? 0;
    });

    try {
      await invoke<string>('download_ytdlp');
      await fetchStatus();
    } catch (e) {
      error.value = String(e);
    } finally {
      unlisten();
      isInstalling.value = false;
    }
  }

  return {
    status,
    updateStatus,
    isLoading,
    isInstalling,
    installProgress,
    error,
    isReady,
    defaultDownloadDir,
    fetchStatus,
    checkUpdate,
    installYtDlp,
  };
});
