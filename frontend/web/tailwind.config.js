/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#FFFBEB',
          100: '#FEF3C7',
          200: '#FDE68A',
          300: '#F7CC5F',
          400: '#F0B429',
          500: '#D99E1B',
          600: '#B78415',
          700: '#926A10',
          800: '#6D500C',
          900: '#493607',
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
        'gradient-brand': 'linear-gradient(135deg, #F0B429 0%, #F7CC5F 100%)',
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
