<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { writeText, writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface ClipboardItem {
  id: string
  content_type: 'Text' | 'Image'
  content: string
  preview: string
  timestamp: string
  is_pinned?: boolean
}

const items = ref<ClipboardItem[]>([])
const copiedId = ref<string | null>(null)
const searchQuery = ref('')
const showScrollTop = ref(false)
const scrollContainer = ref<HTMLElement | null>(null)
let unlisten: UnlistenFn | null = null

onMounted(async () => {
  await loadHistory()
  unlisten = await listen('clipboard-updated', loadHistory)
  
  // Add scroll listener
  if (scrollContainer.value) {
    scrollContainer.value.addEventListener('scroll', handleScroll)
  }
})

onUnmounted(() => {
  if (unlisten) unlisten()
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll)
  }
})

function handleScroll() {
  if (scrollContainer.value) {
    showScrollTop.value = scrollContainer.value.scrollTop > 200
  }
}

function scrollToTop() {
  if (scrollContainer.value) {
    scrollContainer.value.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

async function loadHistory() {
  try {
    const history = await invoke<ClipboardItem[]>('get_clipboard_history')
    items.value = history
  } catch (e) {
    console.error('Failed to load history:', e)
  }
}

const filteredItems = computed(() => {
  let result = items.value
  
  // Search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(item => {
      if (item.content_type === 'Text') {
        return item.content.toLowerCase().includes(query) || 
               item.preview.toLowerCase().includes(query)
      }
      return false // Images can't be searched by content
    })
  }
  
  // Sort: pinned items first, then by timestamp (newest first)
  return result.sort((a, b) => {
    if (a.is_pinned && !b.is_pinned) return -1
    if (!a.is_pinned && b.is_pinned) return 1
    return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  })
})

async function copyItem(item: ClipboardItem, shouldHide = true) {
  try {
    if (item.content_type === 'Text') {
      await writeText(item.content)
    } else {
      const base64Data = item.content
      const byteCharacters = atob(base64Data)
      const byteNumbers = new Array(byteCharacters.length)
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i)
      }
      const byteArray = new Uint8Array(byteNumbers)
      
      const { Image } = await import('@tauri-apps/api/image')
      const image = await Image.fromBytes(byteArray)
      await writeImage(image)
    }
    copiedId.value = item.id

    if (shouldHide) {
      await invoke('paste_and_hide')
    }

    setTimeout(() => (copiedId.value = null), 1500)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}

async function copyOnly(item: ClipboardItem, e: Event) {
  e.stopPropagation()
  await copyItem(item, false)
}

async function deleteItem(id: string, e: Event) {
  e.stopPropagation()
  try {
    await invoke('delete_item', { id })
    await loadHistory()
  } catch (err) {
    console.error('Failed to delete:', err)
  }
}

async function togglePin(item: ClipboardItem, e: Event) {
  e.stopPropagation()
  try {
    await invoke('toggle_pin_item', { id: item.id })
    await loadHistory()
  } catch (err) {
    console.error('Failed to toggle pin:', err)
  }
}

async function clearAll() {
  try {
    await invoke('clear_history')
    items.value = []
  } catch (e) {
    console.error('Failed to clear:', e)
  }
}

async function hideWindow() {
  const win = getCurrentWindow()
  await win.hide()
}
</script>

<template>
  <div class="h-screen flex flex-col bg-gradient-to-b from-primary to-[#16161a]">
    <header class="flex items-center justify-between px-4 py-3.5 bg-secondary border-b border-border select-none">
      <h1 class="text-[15px] font-semibold tracking-tight text-text-primary">剪切板历史</h1>
      <div class="flex gap-1.5">
        <button
          @click="clearAll"
          title="清空历史"
          class="flex items-center justify-center w-7 h-7 rounded-md bg-transparent text-text-secondary hover:bg-hover hover:text-text-primary hover:text-danger transition-all duration-150"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
          </svg>
        </button>
        <button
          @click="hideWindow"
          title="关闭 (Option+Space)"
          class="flex items-center justify-center w-7 h-7 rounded-md bg-transparent text-text-secondary hover:bg-danger hover:text-white transition-all duration-150"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </header>

    <!-- Search Bar -->
    <div class="px-3 py-2 bg-secondary border-b border-border">
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索剪切板内容..."
          class="w-full h-9 pl-9 pr-3 text-[13px] bg-primary border border-border rounded-lg text-text-primary placeholder-muted focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-all"
        />
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="11" cy="11" r="8"/>
          <path d="M21 21l-4.35-4.35"/>
        </svg>
        <button
          v-if="searchQuery"
          @click="searchQuery = ''"
          class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted hover:text-text-primary transition-colors"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </div>

    <div ref="scrollContainer" class="flex-1 overflow-y-auto p-2 relative">
      <!-- Scroll to top button -->
      <button
        v-if="showScrollTop"
        @click="scrollToTop"
        class="fixed bottom-16 right-4 z-50 w-10 h-10 rounded-full bg-accent text-white shadow-lg hover:bg-accent/90 transition-all duration-200 flex items-center justify-center"
        title="滚动到顶部"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 15l-6-6-6 6"/>
        </svg>
      </button>

      <div v-if="filteredItems.length === 0" class="flex flex-col items-center justify-center h-full text-muted text-center px-5 py-10">
        <p class="text-[15px] mb-1.5 text-text-secondary">
          {{ searchQuery ? '未找到匹配的内容' : '暂无剪切板记录' }}
        </p>
        <span class="text-[13px]">{{ searchQuery ? '尝试其他关键词' : '复制文字或图片后会自动记录' }}</span>
      </div>

      <div
        v-for="item in filteredItems"
        :key="item.id"
        @click="copyItem(item)"
        :class="[
          'relative flex flex-col p-3 mb-1.5 rounded-[10px] border cursor-pointer transition-all duration-200 overflow-hidden group',
          copiedId === item.id
            ? 'border-success bg-success/10'
            : 'border-transparent bg-secondary hover:bg-hover hover:border-border hover:-translate-y-0.5',
          item.is_pinned ? 'ring-1 ring-accent/30' : ''
        ]"
      >
        <!-- Pin indicator -->
        <div
          v-if="item.is_pinned"
          class="absolute top-2 left-2 text-accent"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z"/>
          </svg>
        </div>

        <div class="flex-1 min-h-0" :class="{ 'mt-4': item.is_pinned }">
          <img
            v-if="item.content_type === 'Image'"
            :src="`data:image/png;base64,${item.content}`"
            alt="clipboard image"
            class="max-w-full max-h-[200px] rounded-md object-contain bg-black/20"
          />
          <p v-else class="text-[14px] text-text-primary break-words whitespace-pre-wrap leading-relaxed">
            {{ item.preview }}
          </p>
        </div>

        <div class="flex items-center gap-2 mt-2.5 pt-2.5 border-t border-border">
          <span
            :class="[
              'text-[11px] font-medium px-2 py-0.5 rounded bg-primary',
              item.content_type === 'Text' ? 'text-accent' : 'text-success'
            ]"
          >
            {{ item.content_type === 'Text' ? '文字' : '图片' }}
          </span>
          <span class="text-[12px] text-muted flex-1">{{ item.timestamp }}</span>
          
          <!-- Action Buttons -->
          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <!-- Copy Button -->
            <button
              @click="copyOnly(item, $event)"
              title="复制"
              class="flex items-center justify-center w-7 h-7 rounded bg-transparent text-muted hover:bg-accent hover:text-white transition-all duration-150"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
              </svg>
            </button>
            
            <!-- Pin Button -->
            <button
              @click="togglePin(item, $event)"
              :title="item.is_pinned ? '取消固定' : '固定'"
              :class="[
                'flex items-center justify-center w-7 h-7 rounded transition-all duration-150',
                item.is_pinned 
                  ? 'bg-accent text-white' 
                  : 'bg-transparent text-muted hover:bg-accent hover:text-white'
              ]"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z"/>
              </svg>
            </button>
            
            <!-- Delete Button -->
            <button
              @click="deleteItem(item.id, $event)"
              title="删除"
              class="flex items-center justify-center w-7 h-7 rounded bg-transparent text-muted hover:bg-danger hover:text-white transition-all duration-150"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>
        </div>

        <div
          v-if="copiedId === item.id"
          class="absolute top-2.5 right-2.5 text-[11px] font-semibold px-2.5 py-1 rounded bg-success text-white animate-fadeIn"
        >
          已复制
        </div>
      </div>
    </div>

    <footer class="px-4 py-2.5 text-center bg-secondary border-t border-border select-none">
      <span class="text-[12px] text-muted">Cmd + Shift + V 快速打开</span>
    </footer>
  </div>
</template>

<style scoped>
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fadeIn {
  animation: fadeIn 0.2s ease;
}
</style>
