import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import i18n from './i18n'

// 初始化主题
function initTheme() {
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

  const root = document.documentElement
  root.classList.remove('light', 'dark')

  if (theme === 'auto') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    root.classList.add(prefersDark ? 'dark' : 'light')
  } else {
    root.classList.add(theme)
  }
}

// 在应用挂载前初始化主题
initTheme()

const app = createApp(App)
app.use(i18n)
app.mount('#app')
