<script setup lang="ts">
import { ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '../stores/app';
import { useDownloadStore } from '../stores/download';

const appStore = useAppStore();
const downloadStore = useDownloadStore();

const url = ref('');
const downloadFormat = ref<'video' | 'audio'>('video');
const isDownloading = ref(false);

// Initialize download directory
watch(() => appStore.defaultDownloadDir, (dir) => {
  if (dir && !downloadStore.downloadDir) {
    downloadStore.setDownloadDir(dir);
  }
}, { immediate: true });

async function selectFolder() {
  const selected = await open({
    directory: true,
    defaultPath: downloadStore.downloadDir || undefined,
  });
  if (selected) {
    downloadStore.setDownloadDir(selected as string);
  }
}

async function handleAnalyze() {
  if (!url.value.trim()) return;
  await downloadStore.analyzeUrl(url.value.trim());
}

async function handleDownload() {
  if (!url.value.trim() || !downloadStore.downloadDir) return;

  isDownloading.value = true;

  try {
    const title = downloadStore.currentVideoInfo?.title ?? 'Unknown';

    await downloadStore.startDownload({
      url: url.value.trim(),
      output_dir: downloadStore.downloadDir,
      format: downloadFormat.value === 'audio' ? 'audio' : 'best',
      extract_audio: downloadFormat.value === 'audio',
      embed_subs: false,
      playlist_items: null,
    }, title);

    // Clear form after successful start
    url.value = '';
    downloadStore.currentVideoInfo = null;
  } catch (e) {
    console.error('Download failed:', e);
  } finally {
    isDownloading.value = false;
  }
}

function handlePaste() {
  navigator.clipboard.readText().then((text) => {
    url.value = text;
    if (text.trim()) {
      handleAnalyze();
    }
  });
}
</script>

<template>
  <div class="card mb-6">
    <h2 class="text-xl font-semibold text-gray-800 mb-4">새 다운로드</h2>

    <!-- URL Input -->
    <div class="mb-4">
      <div class="flex gap-2">
        <input
          v-model="url"
          type="text"
          class="input flex-1"
          placeholder="여기에 영상 주소를 붙여넣으세요"
          @keyup.enter="handleAnalyze"
          @paste="handleAnalyze"
        />
        <button
          @click="handlePaste"
          class="btn btn-secondary px-4"
          title="클립보드에서 붙여넣기"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Video Info Preview -->
    <div v-if="downloadStore.isAnalyzing" class="mb-4 p-4 bg-gray-50 rounded-lg">
      <div class="flex items-center gap-3">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
        <span class="text-gray-600">영상 정보를 가져오는 중...</span>
      </div>
    </div>

    <div v-if="downloadStore.analyzeError" class="mb-4 p-4 bg-red-50 rounded-lg">
      <p class="text-red-600">{{ downloadStore.analyzeError }}</p>
    </div>

    <div v-if="downloadStore.currentVideoInfo" class="mb-4 p-4 bg-blue-50 rounded-lg">
      <div class="flex gap-4">
        <img
          v-if="downloadStore.currentVideoInfo.thumbnail"
          :src="downloadStore.currentVideoInfo.thumbnail"
          :alt="downloadStore.currentVideoInfo.title"
          class="w-32 h-20 object-cover rounded"
        />
        <div class="flex-1 min-w-0">
          <h3 class="font-semibold text-gray-800 truncate">
            {{ downloadStore.currentVideoInfo.title }}
          </h3>
          <p v-if="downloadStore.currentVideoInfo.uploader" class="text-gray-600 text-sm">
            {{ downloadStore.currentVideoInfo.uploader }}
          </p>
          <p v-if="downloadStore.currentVideoInfo.is_playlist" class="text-blue-600 text-sm mt-1">
            플레이리스트 ({{ downloadStore.currentVideoInfo.playlist_count }}개 영상)
          </p>
        </div>
      </div>
    </div>

    <!-- Format Selection -->
    <div class="mb-4">
      <label class="block text-gray-700 mb-2">다운로드 형식</label>
      <div class="flex gap-3">
        <button
          @click="downloadFormat = 'video'"
          :class="[
            'flex-1 py-3 px-4 rounded-lg border-2 transition-all flex items-center justify-center gap-2',
            downloadFormat === 'video'
              ? 'border-blue-500 bg-blue-50 text-blue-700'
              : 'border-gray-200 hover:border-gray-300 text-gray-600'
          ]"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
          <span class="text-lg">영상</span>
        </button>
        <button
          @click="downloadFormat = 'audio'"
          :class="[
            'flex-1 py-3 px-4 rounded-lg border-2 transition-all flex items-center justify-center gap-2',
            downloadFormat === 'audio'
              ? 'border-blue-500 bg-blue-50 text-blue-700'
              : 'border-gray-200 hover:border-gray-300 text-gray-600'
          ]"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
          </svg>
          <span class="text-lg">음악 (MP3)</span>
        </button>
      </div>
    </div>

    <!-- Download Directory -->
    <div class="mb-6">
      <label class="block text-gray-700 mb-2">저장 위치</label>
      <div class="flex gap-2">
        <input
          :value="downloadStore.downloadDir"
          readonly
          class="input flex-1 bg-gray-50 cursor-pointer"
          @click="selectFolder"
        />
        <button @click="selectFolder" class="btn btn-secondary">
          변경
        </button>
      </div>
    </div>

    <!-- Download Button -->
    <button
      @click="handleDownload"
      :disabled="!url.trim() || !downloadStore.downloadDir || isDownloading"
      class="btn btn-primary w-full text-lg py-4 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      <span v-if="isDownloading" class="flex items-center justify-center gap-2">
        <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
        다운로드 시작 중...
      </span>
      <span v-else>다운로드 시작</span>
    </button>
  </div>
</template>
