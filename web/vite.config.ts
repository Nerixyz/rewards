import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  server: {
    proxy: {
      '/api/': {
        // target: 'https://rewards.nerixyz.de',
        target: 'http://127.0.0.1:8082',
        changeOrigin: true,
      },
    },
  },
});
