<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { writeText, writeImage } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow, getAllWindows } from '@tauri-apps/api/window'

const { t, locale } = useI18n()

// Detect platform
const isMac = navigator.platform.toLowerCase().includes('mac')
const shortcutText = isMac ? 'Cmd + Shift + V' : 'Ctrl + Shift + V'

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
const activeFilter = ref<'all' | 'text' | 'image'>('all')
const showScrollTop = ref(false)
const scrollContainer = ref<HTMLElement | null>(null)
let unlistenClipboard: UnlistenFn | null = null
let unlistenTheme: UnlistenFn | null = null
let unlistenLanguage: UnlistenFn | null = null
let unlistenHistoryCleared: UnlistenFn | null = null

onMounted(async () => {
  // 加载设置
  loadSettings()

  await loadHistory()
  try {
    unlistenClipboard = await listen('clipboard-updated', (event) => {
      console.log('Clipboard updated event received:', event)
      loadHistory()
    })
    console.log('Event listener registered successfully')

    // 监听主题变化事件
    unlistenTheme = await listen('theme-changed', (event) => {
      console.log('Theme changed event received:', event)
      const payload = event.payload as { theme: 'auto' | 'light' | 'dark' }
      applyTheme(payload.theme)
    })

    // 监听语言变化事件
    unlistenLanguage = await listen('language-changed', (event) => {
      console.log('Language changed event received:', event)
      const payload = event.payload as { language: string }
      locale.value = payload.language
    })

    // 监听历史记录清除事件
    unlistenHistoryCleared = await listen('history-cleared', () => {
      console.log('History cleared event received')
      loadHistory()
    })
  } catch (e) {
    console.error('Failed to register event listener:', e)
  }

  // Add scroll listener
  if (scrollContainer.value) {
    scrollContainer.value.addEventListener('scroll', handleScroll)
  }
})

onUnmounted(() => {
  if (unlistenClipboard) unlistenClipboard()
  if (unlistenTheme) unlistenTheme()
  if (unlistenLanguage) unlistenLanguage()
  if (unlistenHistoryCleared) unlistenHistoryCleared()
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll)
  }
})

function loadSettings() {
  const saved = localStorage.getItem('general-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      // 应用主题
      applyTheme(parsed.theme || 'auto')
      // 应用语言
      if (parsed.language) {
        locale.value = parsed.language
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }
}

function applyTheme(theme: 'auto' | 'light' | 'dark') {
  const root = document.documentElement
  root.classList.remove('light', 'dark')

  if (theme === 'auto') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    root.classList.add(prefersDark ? 'dark' : 'light')
  } else {
    root.classList.add(theme)
  }
}

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
    console.log('Loading history...')
    const history = await invoke<ClipboardItem[]>('get_clipboard_history')
    console.log('History loaded:', history)
    items.value = history
    console.log('Items updated:', items.value)
  } catch (e) {
    console.error('Failed to load history:', e)
  }
}

const filteredItems = computed(() => {
  let result = items.value

  // Type filter
  if (activeFilter.value !== 'all') {
    result = result.filter(item => {
      if (activeFilter.value === 'text') return item.content_type === 'Text'
      if (activeFilter.value === 'image') return item.content_type === 'Image'
      return true
    })
  }

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
  if (!confirm(t('historySettings.confirmClear'))) {
    return
  }
  try {
    await invoke('clear_history')
    items.value = []
  } catch (e) {
    console.error('Failed to clear:', e)
    alert(t('common.error'))
  }
}

async function hideWindow() {
  const win = getCurrentWindow()
  await win.hide()
}

async function openSettings() {
  // Get settings window and show it
  const windows = await getAllWindows()
  const settingsWindow = windows.find(w => w.label === 'settings')
  if (settingsWindow) {
    await settingsWindow.show()
    await settingsWindow.setFocus()
    await settingsWindow.center()
  }
}
</script>

<template>
  <div class="h-screen flex flex-col bg-gradient-to-b from-[#f5f5f7] to-[#e8e8ed] dark:from-[#1a1a1e] dark:to-[#16161a]">
    <header class="flex items-center justify-between px-4 py-3.5 bg-white dark:bg-[#232328] border-b border-gray-200 dark:border-[#38383a] select-none">
      <h1 class="text-[15px] font-semibold tracking-tight text-gray-900 dark:text-[#f5f5f7]">{{ t('app.name') }}</h1>
      <div class="flex gap-1.5">
        <button
          @click="openSettings"
          :title="t('menu.settings')"
          class="flex items-center justify-center w-7 h-7 rounded-md bg-transparent text-gray-500 dark:text-[#8e8e93] hover:bg-gray-100 dark:hover:bg-[#2d2d33] hover:text-gray-900 dark:hover:text-[#f5f5f7] transition-all duration-150"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </button>
        <button
          @click="clearAll"
          :title="t('historySettings.clearHistory')"
          class="flex items-center justify-center w-7 h-7 rounded-md bg-transparent text-gray-500 dark:text-[#8e8e93] hover:bg-gray-100 dark:hover:bg-[#2d2d33] hover:text-gray-900 dark:hover:text-[#f5f5f7] hover:text-[#ff453a] transition-all duration-150"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
          </svg>
        </button>
        <button
          @click="hideWindow"
          :title="t('common.close')"
          class="flex items-center justify-center w-7 h-7 rounded-md bg-transparent text-gray-500 dark:text-[#8e8e93] hover:bg-[#ff453a] hover:text-white transition-all duration-150"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </header>

    <!-- Search Bar -->
    <div class="px-3 py-2 bg-white dark:bg-[#232328] border-b border-gray-200 dark:border-[#38383a]">
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('clipboardSettings.searchBox') + '...'"
          class="w-full h-9 pl-9 pr-3 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-[#38383a] rounded-lg text-gray-900 dark:text-[#f5f5f7] placeholder-gray-400 dark:placeholder-[#636366] focus:outline-none focus:border-[#0a84ff] focus:ring-1 focus:ring-[#0a84ff] transition-all"
        />
        <svg
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-[#636366]"
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
          class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-[#636366] hover:text-gray-900 dark:hover:text-[#f5f5f7] transition-colors"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
      </div>
      <!-- Filter Tabs -->
      <div class="flex gap-1 mt-2">
        <button
          @click="activeFilter = 'all'"
          :class="[
            'flex-1 h-7 text-[12px] font-medium rounded-md transition-all duration-150',
            activeFilter === 'all'
              ? 'bg-[#0a84ff] text-white'
              : 'bg-gray-100 dark:bg-[#1a1a1e] text-gray-500 dark:text-[#8e8e93] hover:text-gray-900 dark:hover:text-[#f5f5f7] hover:bg-gray-200 dark:hover:bg-[#2d2d33]'
          ]"
        >
          {{ t('common.all') || '全部' }}
        </button>
        <button
          @click="activeFilter = 'text'"
          :class="[
            'flex-1 h-7 text-[12px] font-medium rounded-md transition-all duration-150',
            activeFilter === 'text'
              ? 'bg-[#0a84ff] text-white'
              : 'bg-gray-100 dark:bg-[#1a1a1e] text-gray-500 dark:text-[#8e8e93] hover:text-gray-900 dark:hover:text-[#f5f5f7] hover:bg-gray-200 dark:hover:bg-[#2d2d33]'
          ]"
        >
          {{ t('common.text') || '文字' }}
        </button>
        <button
          @click="activeFilter = 'image'"
          :class="[
            'flex-1 h-7 text-[12px] font-medium rounded-md transition-all duration-150',
            activeFilter === 'image'
              ? 'bg-[#0a84ff] text-white'
              : 'bg-gray-100 dark:bg-[#1a1a1e] text-gray-500 dark:text-[#8e8e93] hover:text-gray-900 dark:hover:text-[#f5f5f7] hover:bg-gray-200 dark:hover:bg-[#2d2d33]'
          ]"
        >
          {{ t('common.image') || '图片' }}
        </button>
      </div>
    </div>

    <div class="flex-1 relative overflow-hidden">
      <div ref="scrollContainer" class="h-full overflow-y-auto p-2 pb-16">
        <div v-if="filteredItems.length === 0" class="flex flex-col items-center justify-center h-full text-center px-5 py-10">
          <p class="text-[15px] mb-1.5 text-gray-500 dark:text-[#8e8e93]">
            {{ searchQuery ? t('common.noMatch') || '未找到匹配的内容' : t('common.noRecords') || '暂无剪切板记录' }}
          </p>
          <span class="text-[13px] text-gray-400 dark:text-[#636366]">{{ searchQuery ? t('common.tryOther') || '尝试其他关键词' : t('common.autoRecord') || '复制文字或图片后会自动记录' }}</span>
        </div>

        <div
          v-for="item in filteredItems"
          :key="item.id"
          @click="copyItem(item)"
          :class="[
            'relative flex flex-col p-3 mb-1.5 rounded-[10px] border cursor-pointer transition-all duration-200 overflow-hidden group',
            copiedId === item.id
              ? 'border-[#30d158] bg-[#30d158]/10'
              : 'border-transparent bg-white dark:bg-[#232328] hover:bg-gray-100 dark:hover:bg-[#2d2d33] hover:border-gray-200 dark:hover:border-[#38383a] hover:-translate-y-0.5',
            item.is_pinned ? 'ring-1 ring-[#0a84ff]/30' : ''
          ]"
        >
        <!-- Pin indicator -->
        <div
          v-if="item.is_pinned"
          class="absolute top-2 left-2 text-[#0a84ff]"
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
          <p v-else class="text-[14px] text-gray-900 dark:text-[#f5f5f7] break-words whitespace-pre-wrap leading-relaxed">
            {{ item.preview }}
          </p>
        </div>

        <div class="flex items-center gap-2 mt-2.5 pt-2.5 border-t border-gray-200 dark:border-[#38383a]">
          <span
            :class="[
              'text-[11px] font-medium px-2 py-0.5 rounded bg-gray-100 dark:bg-[#1a1a1e]',
              item.content_type === 'Text' ? 'text-[#0a84ff]' : 'text-[#30d158]'
            ]"
          >
            {{ item.content_type === 'Text' ? (t('common.text') || '文字') : (t('common.image') || '图片') }}
          </span>
          <span class="text-[12px] text-gray-400 dark:text-[#636366] flex-1">{{ item.timestamp }}</span>

          <!-- Action Buttons -->
          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <!-- Copy Button -->
            <button
              @click="copyOnly(item, $event)"
              :title="t('common.copy') || '复制'"
              class="flex items-center justify-center w-7 h-7 rounded bg-transparent text-gray-400 dark:text-[#636366] hover:bg-[#0a84ff] hover:text-white transition-all duration-150"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
              </svg>
            </button>

            <!-- Pin Button -->
            <button
              @click="togglePin(item, $event)"
              :title="item.is_pinned ? (t('common.unpin') || '取消固定') : (t('common.pin') || '固定')"
              :class="[
                'flex items-center justify-center w-7 h-7 rounded transition-all duration-150',
                item.is_pinned
                  ? 'bg-[#0a84ff] text-white'
                  : 'bg-transparent text-gray-400 dark:text-[#636366] hover:bg-[#0a84ff] hover:text-white'
              ]"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z"/>
              </svg>
            </button>

            <!-- Delete Button -->
            <button
              @click="deleteItem(item.id, $event)"
              :title="t('common.delete') || '删除'"
              class="flex items-center justify-center w-7 h-7 rounded bg-transparent text-gray-400 dark:text-[#636366] hover:bg-[#ff453a] hover:text-white transition-all duration-150"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>
        </div>

        <div
          v-if="copiedId === item.id"
          class="absolute top-2.5 right-2.5 text-[11px] font-semibold px-2.5 py-1 rounded bg-[#30d158] text-white animate-fadeIn"
        >
          {{ t('common.copied') || '已复制' }}
        </div>
      </div>
      </div>
      <!-- Scroll to top button -->
      <button
        v-if="showScrollTop"
        @click="scrollToTop"
        class="absolute bottom-4 right-4 z-50 w-10 h-10 rounded-full bg-[#0a84ff] text-white shadow-lg hover:bg-[#0a84ff]/90 transition-all duration-200 flex items-center justify-center"
        :title="t('common.scrollTop') || '滚动到顶部'"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 15l-6-6-6 6"/>
        </svg>
      </button>
    </div>

    <footer class="px-4 py-2.5 text-center bg-white dark:bg-[#232328] border-t border-gray-200 dark:border-[#38383a] select-none">
      <span class="text-[12px] text-gray-400 dark:text-[#636366]">{{ shortcutText }} {{ t('common.quickOpen') || '快速打开' }}</span>
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
