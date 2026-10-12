import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

// Fixed port: tauri.conf.json points devUrl here and the dev CSP allows exactly this origin.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**'] },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    // Boot and form tests re-import the whole app; a CPU starved by a parallel cargo build takes
    // them past vitest's 5 s default without anything being wrong.
    testTimeout: 20_000,
    hookTimeout: 20_000,
    restoreMocks: true,
    unstubGlobals: true,
    // Vitest swaps every .css import, ?raw included, for ''; the contrast test reads the real files.
    css: { include: [/\.css\?raw$/] },
  },
});
