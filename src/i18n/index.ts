import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import en from './locales/en.json'

// 从 localStorage 获取保存的语言设置
function getSavedLanguage(): string {
  const saved = localStorage.getItem('general-settings')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      return parsed.language || 'zh-CN'
    } catch (e) {
      console.error('Failed to load language:', e)
    }
  }
  return 'zh-CN'
}

const i18n = createI18n({
  legacy: false,
  locale: getSavedLanguage(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en': en
  }
})

export default i18n
