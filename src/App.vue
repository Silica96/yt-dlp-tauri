<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useAppStore } from './stores/app';
import { useDownloadStore } from './stores/download';
import SetupScreen from './components/SetupScreen.vue';
import AppHeader from './components/AppHeader.vue';
import DownloadForm from './components/DownloadForm.vue';
import DownloadList from './components/DownloadList.vue';

const appStore = useAppStore();
const downloadStore = useDownloadStore();

onMounted(async () => {
  await appStore.fetchStatus();
});

onUnmounted(() => {
  downloadStore.cleanup();
});
</script>

<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Loading state -->
    <div v-if="appStore.isLoading" class="min-h-screen flex items-center justify-center">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
        <p class="text-gray-600">로딩 중...</p>
      </div>
    </div>

    <!-- Setup screen (yt-dlp not installed) -->
    <SetupScreen v-else-if="!appStore.isReady" />

    <!-- Main app -->
    <div v-else class="min-h-screen flex flex-col">
      <AppHeader />

      <main class="flex-1 p-6 max-w-3xl mx-auto w-full">
        <DownloadForm />
        <DownloadList />
      </main>

      <!-- Footer -->
      <footer class="text-center py-4 text-sm text-gray-500">
        <p>yt-dlp를 사용하여 영상을 다운로드합니다</p>
      </footer>
    </div>
  </div>
</template>
