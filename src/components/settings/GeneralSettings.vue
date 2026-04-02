<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t, locale } = useI18n()

interface GeneralSettings {
  loginStart: boolean
  autoCheckUpdate: boolean
  language: string
  theme: 'auto' | 'light' | 'dark'
}

const settings = ref<GeneralSettings>({
  loginStart: false,
  autoCheckUpdate: false,
  language: 'zh-CN',
  theme: 'auto'
})

const permissions = ref({
  accessibility: true,
  diskAccess: true
})

const emit = defineEmits<{
  save: [settings: GeneralSettings]
  themeChange: [theme: 'auto' | 'light' | 'dark']
}>()

onMounted(() => {
  loadSettings()
  checkPermissions()
})

function loadSettings() {
  const saved = localStorage.getItem('general-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      settings.value = { ...settings.value, ...parsed }
      // 同步 i18n locale
      locale.value = settings.value.language
    } catch (e) {
      console.error('Failed to load general settings:', e)
    }
  }
}

function saveSettings() {
  localStorage.setItem('general-settings', JSON.stringify(settings.value))
  emit('save', settings.value)
}

function updateSetting<K extends keyof GeneralSettings>(
  key: K,
  value: GeneralSettings[K]
) {
  settings.value[key] = value
  saveSettings()

  if (key === 'theme') {
    emit('themeChange', value as 'auto' | 'light' | 'dark')
  }

  if (key === 'language') {
    locale.value = value as string
  }
}

function checkPermissions() {
  // TODO: Check actual permissions from system
  // For now, just use default values
  permissions.value = {
    accessibility: true,
    diskAccess: true
  }
}

async function requestAccessibilityPermission() {
  // TODO: Implement actual permission request
  console.log('Requesting accessibility permission')
}

async function requestDiskAccessPermission() {
  // TODO: Implement actual permission request
  console.log('Requesting disk access permission')
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('generalSettings.permissions') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.accessibility') }}</h4>
          <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('generalSettings.accessibilityDesc') }}</p>
        </div>
        <button
          v-if="permissions.accessibility"
          class="text-[13px] text-[#0a84ff] flex items-center gap-1 cursor-default"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          {{ t('generalSettings.authorized') }}
        </button>
        <button
          v-else
          @click="requestAccessibilityPermission"
          class="px-3 py-1.5 text-[13px] font-medium text-white bg-[#0a84ff] rounded-lg hover:bg-[#0a84ff]/90 transition-colors"
        >
          {{ t('generalSettings.authorize') }}
        </button>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.diskAccess') }}</h4>
            <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('generalSettings.diskAccessDesc') }}</p>
          </div>
          <button
            v-if="permissions.diskAccess"
            class="text-[13px] text-[#0a84ff] flex items-center gap-1 cursor-default"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
            {{ t('generalSettings.authorized') }}
          </button>
          <button
            v-else
            @click="requestDiskAccessPermission"
            class="px-3 py-1.5 text-[13px] font-medium text-white bg-[#0a84ff] rounded-lg hover:bg-[#0a84ff]/90 transition-colors"
          >
            {{ t('generalSettings.authorize') }}
          </button>
        </div>
      </div>
    </div>

    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100 pt-4">{{ t('generalSettings.appSettings') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.loginStart') }}</h4>
        </div>
        <button
          @click="updateSetting('loginStart', !settings.loginStart)"
          :class="[
            'w-11 h-6 rounded-full transition-colors relative',
            settings.loginStart ? 'bg-[#0a84ff]' : 'bg-gray-300 dark:bg-gray-600'
          ]"
        >
          <span
            :class="[
              'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
              settings.loginStart ? 'left-6' : 'left-1'
            ]"
          />
        </button>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.autoCheckUpdate') }}</h4>
          </div>
          <button
            @click="updateSetting('autoCheckUpdate', !settings.autoCheckUpdate)"
            :class="[
              'w-11 h-6 rounded-full transition-colors relative',
              settings.autoCheckUpdate ? 'bg-[#0a84ff]' : 'bg-gray-300 dark:bg-gray-600'
            ]"
          >
            <span
              :class="[
                'absolute top-1 w-4 h-4 bg-white rounded-full transition-transform',
                settings.autoCheckUpdate ? 'left-6' : 'left-1'
              ]"
            />
          </button>
        </div>
      </div>
    </div>

    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100 pt-4">{{ t('generalSettings.appearance') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.language') }}</h4>
        </div>
        <select
          :value="settings.language"
          @change="(e) => updateSetting('language', (e.target as HTMLSelectElement).value)"
          class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff]"
        >
          <option value="zh-CN">简体中文</option>
          <option value="en">English</option>
        </select>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('generalSettings.theme') }}</h4>
          </div>
          <select
            :value="settings.theme"
            @change="(e) => updateSetting('theme', (e.target as HTMLSelectElement).value as GeneralSettings['theme'])"
            class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff]"
          >
            <option value="auto">{{ t('generalSettings.themeOptions.auto') }}</option>
            <option value="light">{{ t('generalSettings.themeOptions.light') }}</option>
            <option value="dark">{{ t('generalSettings.themeOptions.dark') }}</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>
