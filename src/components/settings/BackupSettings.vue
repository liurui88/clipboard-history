<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

const { t } = useI18n()

const isExporting = ref(false)
const isImporting = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error'>('success')
const importContent = ref('')
const showImportDialog = ref(false)

function showImport() {
  showImportDialog.value = true
  importContent.value = ''
  message.value = ''
}

function cancelImport() {
  showImportDialog.value = false
  importContent.value = ''
}

async function exportData() {
  isExporting.value = true
  message.value = ''

  try {
    // Get clipboard history from backend
    const history = await invoke('get_clipboard_history')

    // Convert to JSON string
    const data = JSON.stringify(history, null, 2)

    // Open save dialog
    const defaultFileName = `clipboard-backup-${new Date().toISOString().split('T')[0]}.json`
    const filePath = await save({
      filters: [
        {
          name: 'JSON Files',
          extensions: ['json']
        }
      ],
      defaultPath: defaultFileName,
    })

    if (!filePath) {
      // User cancelled the dialog
      isExporting.value = false
      return
    }

    // Write file using Tauri FS API
    await writeTextFile(filePath, data)

    message.value = t('backupSettings.exportSuccess')
    messageType.value = 'success'
  } catch (err) {
    console.error('Failed to export data:', err)
    message.value = t('backupSettings.exportError') + '：' + (err as Error).message
    messageType.value = 'error'
  } finally {
    isExporting.value = false
    setTimeout(() => {
      message.value = ''
    }, 3000)
  }
}

async function confirmImport() {
  if (!importContent.value.trim()) {
    message.value = t('backupSettings.emptyImport')
    messageType.value = 'error'
    return
  }

  isImporting.value = true
  message.value = ''

  try {
    // Parse and validate JSON data
    JSON.parse(importContent.value)

    // TODO: Import data to backend
    // await invoke('import_history', { data: parsedData })

    message.value = t('backupSettings.importSuccess')
    messageType.value = 'success'
    showImportDialog.value = false
    importContent.value = ''
  } catch (err) {
    console.error('Failed to import data:', err)
    message.value = t('backupSettings.importError')
    messageType.value = 'error'
  } finally {
    isImporting.value = false
    setTimeout(() => {
      message.value = ''
    }, 3000)
  }
}

function handleFileUpload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    importContent.value = e.target?.result as string
  }
  reader.readAsText(file)
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('backupSettings.title') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('backupSettings.export') }}</h4>
          <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('backupSettings.exportDesc') }}</p>
        </div>
        <button
          @click="exportData"
          :disabled="isExporting"
          class="px-4 py-2 text-[13px] font-medium text-white bg-[#0a84ff] rounded-lg hover:bg-[#0a84ff]/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isExporting ? t('backupSettings.exporting') : t('backupSettings.exportButton') }}
        </button>
      </div>

      <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('backupSettings.import') }}</h4>
            <p class="text-[12px] text-gray-500 dark:text-gray-400 mt-0.5">{{ t('backupSettings.importDesc') }}</p>
          </div>
          <button
            @click="showImport"
            :disabled="isImporting"
            class="px-4 py-2 text-[13px] font-medium text-[#0a84ff] border border-[#0a84ff] rounded-lg hover:bg-[#0a84ff]/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {{ isImporting ? t('backupSettings.importing') : t('backupSettings.importButton') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Import Dialog -->
    <div v-if="showImportDialog" class="bg-white dark:bg-[#232328] rounded-xl p-4 shadow-sm space-y-4">
      <h4 class="text-[14px] font-medium text-gray-900 dark:text-gray-100">{{ t('backupSettings.importTitle') }}</h4>

      <div class="space-y-2">
        <label class="text-[12px] text-gray-500 dark:text-gray-400">{{ t('backupSettings.importInstruction') }}</label>
        <input
          type="file"
          accept=".json"
          @change="handleFileUpload"
          class="block w-full text-[13px] text-gray-700 dark:text-gray-300 file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:text-[13px] file:font-medium file:bg-[#0a84ff] file:text-white hover:file:bg-[#0a84ff]/90"
        />
      </div>

      <textarea
        v-model="importContent"
        rows="6"
        :placeholder="t('backupSettings.importPlaceholder')"
        class="w-full px-3 py-2 text-[13px] bg-gray-100 dark:bg-[#1a1a1e] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-[#0a84ff] font-mono"
      />

      <div class="flex gap-2">
        <button
          @click="cancelImport"
          class="flex-1 px-4 py-2 text-[13px] font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-[#1a1a1e] rounded-lg hover:bg-gray-200 dark:hover:bg-[#2d2d33] transition-colors"
        >
          {{ t('backupSettings.cancel') }}
        </button>
        <button
          @click="confirmImport"
          :disabled="isImporting"
          class="flex-1 px-4 py-2 text-[13px] font-medium text-white bg-[#0a84ff] rounded-lg hover:bg-[#0a84ff]/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isImporting ? t('backupSettings.importing') : t('backupSettings.confirmImport') }}
        </button>
      </div>
    </div>

    <div v-if="message" class="text-center">
      <span
        class="text-[13px]"
        :class="messageType === 'success' ? 'text-[#30d158]' : 'text-[#ff453a]'"
      >
        {{ message }}
      </span>
    </div>
  </div>
</template>
