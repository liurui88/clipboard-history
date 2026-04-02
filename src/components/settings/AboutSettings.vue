<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const appInfo = {
  name: 'Clipboard History',
  version: '0.1.0',
}
</script>

<template>
  <div class="space-y-6">
    <h3 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100">{{ t('aboutSettings.title') }}</h3>

    <div class="bg-white dark:bg-[#232328] rounded-xl p-6 shadow-sm text-center">
      <div class="w-20 h-20 mx-auto mb-4 rounded-2xl bg-[#0a84ff]/10 flex items-center justify-center">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-[#0a84ff]">
          <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
        </svg>
      </div>
      <h4 class="text-[18px] font-semibold text-gray-900 dark:text-gray-100 mb-1">{{ t('app.name') }}</h4>
      <p class="text-[14px] text-gray-500 dark:text-gray-400 mb-4">{{ appInfo.version }}</p>
      <p class="text-[13px] text-gray-600 dark:text-gray-400 leading-relaxed">
        {{ t('aboutSettings.description') }}
      </p>
      <div class="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
        <p class="text-[12px] text-gray-400 dark:text-gray-500">
          {{ t('aboutSettings.copyright') }}
        </p>
      </div>
    </div>
  </div>
</template>
