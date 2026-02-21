/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#EDE9FE',
          100: '#D4CAFE',
          200: '#B8A9E8',
          300: '#8B73E6',
          400: '#7A63D9',
          500: '#6E56CF',
          600: '#5B45B0',
          700: '#4A3791',
          800: '#3A2B72',
          900: '#2A1F53',
        },
        dark: {
          50: '#FAFAFA',
          100: '#E4E4E7',
          200: '#D4D4D8',
          300: '#A1A1AA',
          400: '#71717A',
          500: '#52525B',
          600: '#2A2A2E',
          700: '#1A1A1D',
          800: '#111113',
          900: '#0A0A0B',
        },
      },
      backgroundImage: {
        'gradient-brand': 'linear-gradient(135deg, #6E56CF 0%, #8B73E6 100%)',
        'gradient-dark': 'linear-gradient(180deg, #0A0A0B 0%, #111113 100%)',
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [],
};
