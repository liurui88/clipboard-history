<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface HistorySettings {
  retentionDays: number
}

const settings = ref<HistorySettings>({
  retentionDays: 0
})

const isClearing = ref(false)
const showClearSuccess = ref(false)

const emit = defineEmits<{
  save: [settings: HistorySettings]
}>()

onMounted(() => {
  loadSettings()
})

function loadSettings() {
  const saved = localStorage.getItem('history-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      settings.value = { ...settings.value, ...parsed }
    } catch (e) {
      console.error('Failed to load history settings:', e)
    }
  }
}

function saveSettings() {
  localStorage.setItem('history-settings', JSON.stringify(settings.value))
  emit('save', settings.value)
}

function updateRetentionDays(days: number) {
  if (days < 0) days = 0
  settings.value.retentionDays = days
  saveSettings()
}

async function clearHistory() {
  if (!confirm(t('historySettings.confirmClear'))) {
    return
  }

  isClearing.value = true
  try {
    await invoke('clear_history')
    showClearSuccess.value = true
    setTimeout(() => {
      showClearSuccess.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to clear history:', err)
    alert(t('common.error'))
  } finally {
    isClearing.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('historySettings.title') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('historySettings.retention') }}</h4>
          <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('historySettings.retentionDesc') }}</p>
        </div>
        <div class="flex items-center gap-2">
          <input
            :value="settings.retentionDays"
            @change="(e) => updateRetentionDays(parseInt((e.target as HTMLInputElement).value) || 0)"
            type="number"
            min="0"
            class="w-16 px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff] text-center"
          />
          <span class="text-[13px] text-gray-600 dark:text-gray-400">{{ t('historySettings.days') }}</span>
        </div>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('historySettings.clearHistory') }}</h4>
            <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('historySettings.clearHistoryDesc') }}</p>
          </div>
        </div>
        <button
          @click="clearHistory"
          :disabled="isClearing"
          class="w-full py-2.5 text-[14px] font-medium text-[#ff453a] border border-[#ff453a] rounded-lg hover:bg-[#ff453a]/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isClearing ? t('historySettings.clearing') : t('historySettings.clearButton') }}
        </button>
      </div>

      <div v-if="showClearSuccess" class="text-center">
        <span class="text-[13px] text-[#30d158]">{{ t('historySettings.clearSuccess') }}</span>
      </div>
    </div>
  </div>
</template>
