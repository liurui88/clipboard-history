<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface ShortcutSettings {
  shortcutClipboard: string
  shortcutSettings: string
}

const settings = ref<ShortcutSettings>({
  shortcutClipboard: 'CommandOrControl+Shift+V',
  shortcutSettings: 'CommandOrControl+Shift+,'
})

const isRecording = ref<'clipboard' | 'settings' | null>(null)
const recordedKeys = ref<string[]>([])

const emit = defineEmits<{
  save: [settings: ShortcutSettings]
}>()

onMounted(() => {
  loadSettings()
  window.addEventListener('keydown', handleKeyDown)
  window.addEventListener('keyup', handleKeyUp)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  window.removeEventListener('keyup', handleKeyUp)
})

function loadSettings() {
  const saved = localStorage.getItem('shortcut-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      settings.value = { ...settings.value, ...parsed }
    } catch (e) {
      console.error('Failed to load shortcut settings:', e)
    }
  }
}

function saveSettings() {
  localStorage.setItem('shortcut-settings', JSON.stringify(settings.value))
  emit('save', settings.value)
}

function formatShortcut(shortcut: string): string {
  const isMac = navigator.platform.toLowerCase().includes('mac')
  if (isMac) {
    return shortcut
      .replace('CommandOrControl', '⌘')
      .replace('Shift', '⇧')
      .replace('Alt', '⌥')
      .replace('Control', '⌃')
      .replace('Meta', '⌘')
  } else {
    return shortcut
      .replace('CommandOrControl', 'Ctrl')
      .replace('Shift', 'Shift')
      .replace('Alt', 'Alt')
      .replace('Control', 'Ctrl')
  }
}

function startRecording(type: 'clipboard' | 'settings') {
  isRecording.value = type
  recordedKeys.value = []
}

function stopRecording() {
  if (isRecording.value && recordedKeys.value.length > 0) {
    const shortcut = recordedKeys.value.join('+')
    if (isRecording.value === 'clipboard') {
      settings.value.shortcutClipboard = shortcut
    } else {
      settings.value.shortcutSettings = shortcut
    }
    saveSettings()
  }
  isRecording.value = null
  recordedKeys.value = []
}

function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return

  e.preventDefault()

  const keys: string[] = []

  if (e.metaKey) keys.push('CommandOrControl')
  if (e.ctrlKey && !e.metaKey) keys.push('Control')
  if (e.altKey) keys.push('Alt')
  if (e.shiftKey) keys.push('Shift')

  if (e.key && e.key !== 'Meta' && e.key !== 'Control' && e.key !== 'Alt' && e.key !== 'Shift') {
    keys.push(e.key.toUpperCase())
  }

  recordedKeys.value = keys
}

function handleKeyUp(e: KeyboardEvent) {
  if (!isRecording.value) return

  e.preventDefault()

  // Stop recording when all modifier keys are released
  if (!e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) {
    stopRecording()
  }
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('shortcutSettings.title') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('shortcutSettings.openClipboard') }}</h4>
        </div>
        <button
          @click="startRecording('clipboard')"
          class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 hover:border-[#0a84ff] transition-colors font-mono min-w-[120px]"
          :class="{ 'border-[#0a84ff] ring-2 ring-[#0a84ff]/20': isRecording === 'clipboard' }"
        >
          {{ isRecording === 'clipboard' ? t('shortcutSettings.recording') : formatShortcut(settings.shortcutClipboard) }}
        </button>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('shortcutSettings.openSettings') }}</h4>
          </div>
          <button
            @click="startRecording('settings')"
            class="px-3 py-1.5 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 hover:border-[#0a84ff] transition-colors font-mono min-w-[120px]"
            :class="{ 'border-[#0a84ff] ring-2 ring-[#0a84ff]/20': isRecording === 'settings' }"
          >
            {{ isRecording === 'settings' ? t('shortcutSettings.recording') : formatShortcut(settings.shortcutSettings) }}
          </button>
        </div>
      </div>
    </div>

    <div class="text-[12px] text-gray-500 dark:text-gray-400">
      <p>{{ t('shortcutSettings.instruction') }}</p>
      <p class="mt-1">{{ t('shortcutSettings.supportedKeys') }}</p>
    </div>
  </div>
</template>
