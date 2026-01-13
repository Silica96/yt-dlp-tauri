<script setup lang="ts">
import { computed } from 'vue';
import { useAppStore } from '../stores/app';

const appStore = useAppStore();

const progressWidth = computed(() => `${appStore.installProgress}%`);
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 p-8">
    <div class="card max-w-md w-full text-center">
      <div class="mb-8">
        <div class="w-20 h-20 mx-auto mb-4 bg-blue-100 rounded-full flex items-center justify-center">
          <svg class="w-10 h-10 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </div>
        <h1 class="text-2xl font-bold text-gray-800 mb-2">영상 다운로더</h1>
        <p class="text-gray-600 text-lg">
          시작하기 전에 yt-dlp를 설치해야 합니다.
        </p>
      </div>

      <div v-if="appStore.isInstalling" class="mb-6">
        <div class="progress-bar mb-2">
          <div class="progress-bar-fill" :style="{ width: progressWidth }"></div>
        </div>
        <p class="text-gray-600">설치 중... {{ Math.round(appStore.installProgress) }}%</p>
      </div>

      <div v-if="appStore.error" class="mb-6 p-4 bg-red-50 rounded-lg">
        <p class="text-red-600">{{ appStore.error }}</p>
      </div>

      <button
        v-if="!appStore.isInstalling"
        @click="appStore.installYtDlp"
        class="btn btn-primary w-full text-lg py-4"
        :disabled="appStore.isInstalling"
      >
        yt-dlp 설치하기
      </button>

      <p class="mt-4 text-sm text-gray-500">
        yt-dlp는 유튜브 등 다양한 사이트에서<br>
        영상을 다운로드할 수 있는 도구입니다.
      </p>
    </div>
  </div>
</template>
