<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ClipboardHistory from './components/ClipboardHistory.vue'
import Settings from './components/Settings.vue'

type Page = 'history' | 'settings'

const currentPage = ref<Page>('history')

onMounted(() => {
  // Check URL parameter to determine which page to show
  const urlParams = new URLSearchParams(window.location.search)
  const page = urlParams.get('page')
  if (page === 'settings') {
    currentPage.value = 'settings'
  }
})
</script>

<template>
  <div class="h-screen w-screen overflow-hidden">
    <ClipboardHistory v-if="currentPage === 'history'" />
    <Settings v-else-if="currentPage === 'settings'" />
  </div>
</template>
