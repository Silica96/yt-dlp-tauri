<script setup lang="ts">
import { ref } from 'vue';
import { useAppStore } from '../stores/app';

const appStore = useAppStore();
const showSettings = ref(false);

async function handleCheckUpdate() {
  await appStore.checkUpdate();
  if (appStore.updateStatus?.update_available) {
    if (confirm(`새 버전이 있습니다: ${appStore.updateStatus.latest_version}\n업데이트하시겠습니까?`)) {
      await appStore.installYtDlp();
    }
  } else {
    alert('현재 최신 버전입니다.');
  }
}
</script>

<template>
  <header class="bg-white border-b border-gray-200 px-6 py-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 bg-blue-500 rounded-lg flex items-center justify-center">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        <div>
          <h1 class="text-xl font-bold text-gray-800">영상 다운로더</h1>
          <p v-if="appStore.status?.ytdlp_version" class="text-sm text-gray-500">
            yt-dlp {{ appStore.status.ytdlp_version }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          @click="handleCheckUpdate"
          class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
          title="업데이트 확인"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
        <button
          @click="showSettings = !showSettings"
          class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
          title="설정"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Settings dropdown -->
    <div v-if="showSettings" class="mt-4 p-4 bg-gray-50 rounded-lg">
      <h3 class="font-semibold text-gray-700 mb-3">앱 정보</h3>
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-gray-600">yt-dlp 버전</span>
          <span class="text-gray-800">{{ appStore.status?.ytdlp_version ?? '설치되지 않음' }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-600">ffmpeg 설치</span>
          <span :class="appStore.status?.ffmpeg_installed ? 'text-green-600' : 'text-yellow-600'">
            {{ appStore.status?.ffmpeg_installed ? '설치됨' : '설치되지 않음' }}
          </span>
        </div>
      </div>
    </div>
  </header>
</template>
