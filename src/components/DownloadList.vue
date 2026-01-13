<script setup lang="ts">
import { computed } from 'vue';
import { useDownloadStore } from '../stores/download';

const downloadStore = useDownloadStore();

const hasDownloads = computed(() => downloadStore.downloads.length > 0);

function getStatusText(status: string): string {
  switch (status) {
    case 'pending': return '대기 중';
    case 'downloading': return '다운로드 중';
    case 'processing': return '변환 중';
    case 'completed': return '완료';
    case 'error': return '오류';
    default: return status;
  }
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'pending': return 'text-gray-500';
    case 'downloading': return 'text-blue-500';
    case 'processing': return 'text-yellow-500';
    case 'completed': return 'text-green-500';
    case 'error': return 'text-red-500';
    default: return 'text-gray-500';
  }
}
</script>

<template>
  <div v-if="hasDownloads" class="card">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-gray-800">다운로드 목록</h2>
      <button
        v-if="downloadStore.completedDownloads.length > 0"
        @click="downloadStore.clearCompleted"
        class="text-sm text-blue-600 hover:text-blue-700"
      >
        완료 항목 삭제
      </button>
    </div>

    <div class="space-y-3">
      <div
        v-for="download in downloadStore.downloads"
        :key="download.id"
        class="p-4 bg-gray-50 rounded-lg"
      >
        <div class="flex items-start gap-3">
          <!-- Status Icon -->
          <div class="mt-1">
            <!-- Downloading spinner -->
            <div
              v-if="download.status === 'downloading' || download.status === 'processing'"
              class="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"
            ></div>
            <!-- Completed check -->
            <svg
              v-else-if="download.status === 'completed'"
              class="w-5 h-5 text-green-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <!-- Error X -->
            <svg
              v-else-if="download.status === 'error'"
              class="w-5 h-5 text-red-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            <!-- Pending clock -->
            <svg
              v-else
              class="w-5 h-5 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between mb-1">
              <h3 class="font-medium text-gray-800 truncate pr-4">
                {{ download.title }}
              </h3>
              <button
                @click="downloadStore.removeDownload(download.id)"
                class="text-gray-400 hover:text-gray-600 flex-shrink-0"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Progress bar for downloading/processing -->
            <div
              v-if="download.status === 'downloading' || download.status === 'processing'"
              class="mb-2"
            >
              <div class="progress-bar h-2">
                <div
                  class="progress-bar-fill"
                  :style="{ width: `${download.progress}%` }"
                ></div>
              </div>
            </div>

            <!-- Status and info -->
            <div class="flex items-center gap-4 text-sm">
              <span :class="getStatusColor(download.status)">
                {{ getStatusText(download.status) }}
              </span>
              <span v-if="download.progress > 0 && download.status !== 'completed'" class="text-gray-500">
                {{ Math.round(download.progress) }}%
              </span>
              <span v-if="download.speed" class="text-gray-500">
                {{ download.speed }}
              </span>
              <span v-if="download.eta" class="text-gray-500">
                남은 시간: {{ download.eta }}
              </span>
            </div>

            <!-- Error message -->
            <p v-if="download.error" class="text-red-500 text-sm mt-1">
              {{ download.error }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
