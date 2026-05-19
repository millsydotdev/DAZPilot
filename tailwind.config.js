/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        void: '#0a0a0a',
        surface: {
          DEFAULT: '#111111',
          elevated: '#1a1a1a',
        },
        accent: {
          DEFAULT: '#ef4444',
          hover: '#f87171',
          dark: '#dc2626',
          muted: 'rgba(239, 68, 68, 0.12)',
        },
        cyan: {
          DEFAULT: '#22d3ee',
          muted: 'rgba(34, 211, 238, 0.1)',
        },
        border: {
          subtle: 'rgba(255, 255, 255, 0.08)',
        },
        primary: {
          DEFAULT: '#ef4444',
          hover: '#f87171',
          foreground: '#ffffff',
        },
        background: '#0a0a0a',
        muted: '#71717a',
        success: '#22c55e',
        warning: '#f59e0b',
        error: '#ef4444',
      },
      spacing: {
        xs: '4px',
        sm: '8px',
        md: '16px',
        lg: '24px',
        xl: '32px',
      },
      borderRadius: {
        sm: '4px',
        md: '8px',
        lg: '12px',
        full: '9999px',
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        display: ['Outfit', 'Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'source-code-pro', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
      },
      boxShadow: {
        'glow-red': '0 0 24px rgba(239, 68, 68, 0.15)',
        'glow-red-sm': '0 0 12px rgba(239, 68, 68, 0.25)',
        'glow-cyan': '0 0 16px rgba(34, 211, 238, 0.2)',
      },
      animation: {
        'fade-in': 'fadeIn 0.4s ease-out',
        'pulse-red': 'pulseRed 2s ease-in-out infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0', transform: 'translateY(4px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        pulseRed: {
          '0%, 100%': { opacity: '0.4' },
          '50%': { opacity: '0.8' },
        },
      },
    },
  },
  plugins: [],
};
