import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'url';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';

// Load environment files based on mode
const mode = process.env.NODE_ENV || 'development';
if (mode === 'production') {
  // For production builds, load production environment first
  dotenv.config({ path: '.env.production' });
  dotenv.config({ path: '../../.env.production' });
} else {
  // For development, load local .env files first, then fallback to project root
  dotenv.config({ path: '.env' });
  dotenv.config({ path: '../../.env' });
}

export default defineConfig({
  build: {
    emptyOutDir: true,
    target: 'es2020',
    rollupOptions: {
      output: {
        format: 'iife',
        inlineDynamicImports: true
      }
    }
  },
  server: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4943",
        changeOrigin: true,
      },
    },
  },
  publicDir: "assets",
  plugins: [
    react(),
    environment("all", { prefix: "CANISTER_" }),
    environment("all", { prefix: "DFX_" }),
  ],
  resolve: {
    alias: [
      {
        find: "declarations",
        replacement: fileURLToPath(
          new URL("../declarations", import.meta.url)
        ),
      },
      {
        find: "@",
        replacement: fileURLToPath(new URL("./src", import.meta.url)),
      },
    ],
  },
  define: {
    global: 'globalThis',
  },
});