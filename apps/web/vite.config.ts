import { defineConfig } from 'vitest/config';
import preact from '@preact/preset-vite';
import tailwindcss from '@tailwindcss/vite';
export default defineConfig({
  plugins: [preact(), tailwindcss()],
  server: {
    proxy: {
      '/api': process.env.API_PROXY_TARGET || 'http://127.0.0.1:3000',
      '/health': process.env.API_PROXY_TARGET || 'http://127.0.0.1:3000',
    },
  },
  test: { environment: 'jsdom', setupFiles: ['./tests/setup.ts'] },
});
