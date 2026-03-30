/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#1a1a1e',
        secondary: '#232328',
        hover: '#2d2d33',
        accent: '#0a84ff',
        'accent-hover': '#409cff',
        success: '#30d158',
        danger: '#ff453a',
        border: '#38383a',
        muted: '#636366',
        text: {
          primary: '#f5f5f7',
          secondary: '#8e8e93',
        },
      },
      fontFamily: {
        sans: ['DM Sans', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
