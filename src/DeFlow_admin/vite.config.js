import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'url';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';

// Load local .env file first, then fallback to project root
dotenv.config({ path: '.env' });
dotenv.config({ path: '../../.env' });

export default defineConfig({
  build: {
    emptyOutDir: true,
    target: 'es2020',
    rollupOptions: {
      output: {
        format: 'iife',
        inlineDynamicImports: true
      }
    },
    // SECURITY: Minify for production
    minify: 'esbuild' // Use esbuild minifier (built-in)
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
    environment("all", { prefix: "VITE_" }),
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
    // Explicitly define environment variables for build time
    'process.env.VITE_OWNER_PRINCIPAL': JSON.stringify(process.env.VITE_OWNER_PRINCIPAL),
    'process.env.VITE_CANISTER_ID_DEFLOW_POOL': JSON.stringify(process.env.VITE_CANISTER_ID_DEFLOW_POOL),
    'process.env.VITE_CANISTER_ID_DEFLOW_BACKEND': JSON.stringify(process.env.VITE_CANISTER_ID_DEFLOW_BACKEND),
    'process.env.VITE_CANISTER_ID_DEFLOW_FRONTEND': JSON.stringify(process.env.VITE_CANISTER_ID_DEFLOW_FRONTEND),
    'process.env.VITE_CANISTER_ID_DEFLOW_ADMIN': JSON.stringify(process.env.VITE_CANISTER_ID_DEFLOW_ADMIN),
    'process.env.VITE_INTERNET_IDENTITY_CANISTER_ID': JSON.stringify(process.env.VITE_INTERNET_IDENTITY_CANISTER_ID),
    'process.env.DFX_NETWORK': JSON.stringify(process.env.DFX_NETWORK),
  },
});