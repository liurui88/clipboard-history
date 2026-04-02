<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface ClipboardSettings {
  windowPosition: 'remember' | 'center' | 'last'
  copySound: boolean
  searchPosition: 'top' | 'bottom'
  autoFocus: boolean
  autoClear: boolean
}

const settings = ref<ClipboardSettings>({
  windowPosition: 'remember',
  copySound: false,
  searchPosition: 'top',
  autoFocus: false,
  autoClear: false
})

const emit = defineEmits<{
  save: [settings: ClipboardSettings]
}>()

onMounted(() => {
  loadSettings()
})

function loadSettings() {
  const saved = localStorage.getItem('clipboard-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      settings.value = { ...settings.value, ...parsed }
    } catch (e) {
      console.error('Failed to load clipboard settings:', e)
    }
  }
}

function saveSettings() {
  localStorage.setItem('clipboard-settings', JSON.stringify(settings.value))
  emit('save', settings.value)
}

function updateSetting<K extends keyof ClipboardSettings>(
  key: K,
  value: ClipboardSettings[K]
) {
  settings.value[key] = value
  saveSettings()
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.windowPosition') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.windowPosition') }}</h4>
        </div>
        <select
          :value="settings.windowPosition"
          @change="(e) => updateSetting('windowPosition', (e.target as HTMLSelectElement).value as ClipboardSettings['windowPosition'])"
          class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff]"
        >
          <option value="remember">{{ t('clipboardSettings.windowPositionOptions.remember') }}</option>
          <option value="center">{{ t('clipboardSettings.windowPositionOptions.center') }}</option>
          <option value="last">{{ t('clipboardSettings.windowPositionOptions.last') }}</option>
        </select>
      </div>
    </div>

    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100 pt-4">{{ t('clipboardSettings.copySound') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.copySound') }}</h4>
          <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('clipboardSettings.copySoundDesc') }}</p>
        </div>
        <button
          @click="updateSetting('copySound', !settings.copySound)"
          :class="[
            'w-11 h-6 rounded-full transition-colors relative',
            settings.copySound ? 'bg-[#0a84ff]' : 'bg-gray-300 dark:bg-gray-600'
          ]"
        >
          <span
            :class="[
              'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
              settings.copySound ? 'left-6' : 'left-1'
            ]"
          />
        </button>
      </div>
    </div>

    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100 pt-4">{{ t('clipboardSettings.searchBox') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.searchPosition') }}</h4>
        </div>
        <select
          :value="settings.searchPosition"
          @change="(e) => updateSetting('searchPosition', (e.target as HTMLSelectElement).value as ClipboardSettings['searchPosition'])"
          class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff]"
        >
          <option value="top">{{ t('clipboardSettings.searchPositionOptions.top') }}</option>
          <option value="bottom">{{ t('clipboardSettings.searchPositionOptions.bottom') }}</option>
        </select>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.autoFocus') }}</h4>
          </div>
          <button
            @click="updateSetting('autoFocus', !settings.autoFocus)"
            :class="[
              'w-11 h-6 rounded-full transition-colors relative',
              settings.autoFocus ? 'bg-[#0a84ff]' : 'bg-gray-300 dark:bg-gray-600'
            ]"
          >
            <span
              :class="[
                'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
                settings.autoFocus ? 'left-6' : 'left-1'
              ]"
            />
          </button>
        </div>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('clipboardSettings.autoClear') }}</h4>
          </div>
          <button
            @click="updateSetting('autoClear', !settings.autoClear)"
            :class="[
              'w-11 h-6 rounded-full transition-colors relative',
              settings.autoClear ? 'bg-[#0a84ff]' : 'bg-gray-300 dark:bg-gray-600'
            ]"
          >
            <span
              :class="[
                'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
                settings.autoClear ? 'left-6' : 'left-1'
              ]"
            />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
