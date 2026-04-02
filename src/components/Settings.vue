<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import ClipboardSettings from './settings/ClipboardSettings.vue'
import HistorySettings from './settings/HistorySettings.vue'
import GeneralSettings from './settings/GeneralSettings.vue'
import ShortcutSettings from './settings/ShortcutSettings.vue'
import BackupSettings from './settings/BackupSettings.vue'
import AboutSettings from './settings/AboutSettings.vue'

const { t } = useI18n()

type SettingTab = 'clipboard' | 'history' | 'general' | 'shortcut' | 'backup' | 'about'

const activeTab = ref<SettingTab>('clipboard')

const tabs = computed(() => [
  { id: 'clipboard' as SettingTab, label: t('settings.tabs.clipboard'), icon: 'clipboard' },
  { id: 'history' as SettingTab, label: t('settings.tabs.history'), icon: 'history' },
  { id: 'general' as SettingTab, label: t('settings.tabs.general'), icon: 'settings' },
  { id: 'shortcut' as SettingTab, label: t('settings.tabs.shortcut'), icon: 'keyboard' },
  { id: 'backup' as SettingTab, label: t('settings.tabs.backup'), icon: 'backup' },
  { id: 'about' as SettingTab, label: t('settings.tabs.about'), icon: 'info' },
])

// 初始化主题
onMounted(() => {
  loadAndApplyTheme()
})

function loadAndApplyTheme() {
  const saved = localStorage.getItem('general-settings')
  let theme: 'auto' | 'light' | 'dark' = 'auto'
  
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      theme = parsed.theme || 'auto'
    } catch (e) {
      console.error('Failed to load theme:', e)
    }
  }
  
  applyTheme(theme)
}

function getIcon(name: string) {
  const icons: Record<string, string> = {
    clipboard: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2',
    history: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
    settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
    keyboard: 'M4 6h16M4 10h16M4 14h16M4 18h16M8 6v12M12 6v12M16 6v12',
    backup: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4',
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  }
  return icons[name] || ''
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
</script>

<template>
  <div class="h-full flex bg-[#f5f5f7] dark:bg-[#1a1a1e]">
    <!-- Left Sidebar -->
    <div class="w-52 bg-[#e8e8ed] dark:bg-[#232328] flex flex-col select-none">
      <div class="p-4">
        <h2 class="text-[13px] font-semibold text-gray-500 dark:text-gray-400 px-3 mb-2">{{ t('settings.title') }}</h2>
      </div>
      <nav class="flex-1 px-3 pb-4 space-y-1">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="[
            'w-full flex items-center gap-3 px-3 py-2 text-[13px] rounded-lg transition-all duration-150',
            activeTab === tab.id
              ? 'bg-[#0a84ff] text-white'
              : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#2d2d33]'
          ]"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path :d="getIcon(tab.icon)"/>
          </svg>
          {{ tab.label }}
        </button>
      </nav>
    </div>

    <!-- Right Content -->
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-xl mx-auto p-8">
        <ClipboardSettings v-if="activeTab === 'clipboard'" />
        <HistorySettings v-else-if="activeTab === 'history'" />
        <GeneralSettings 
          v-else-if="activeTab === 'general'" 
          @theme-change="applyTheme"
        />
        <ShortcutSettings v-else-if="activeTab === 'shortcut'" />
        <BackupSettings v-else-if="activeTab === 'backup'" />
        <AboutSettings v-else-if="activeTab === 'about'" />
      </div>
    </div>
  </div>
</template>
