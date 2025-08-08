import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'url';
import environment from 'vite-plugin-environment';
import { nodePolyfills } from 'vite-plugin-node-polyfills';
import dotenv from 'dotenv';

dotenv.config({ path: '../../.env' });

export default defineConfig({
  build: {
    emptyOutDir: true,
    target: 'es2015',
    rollupOptions: {
      external: [],
      output: {
        format: 'iife',
        inlineDynamicImports: true
      },
      onwarn(warning, warn) {
        // Ignore lit-html warnings
        if (warning.code === 'UNRESOLVED_IMPORT' && warning.source === 'lit-html') {
          return;
        }
        warn(warning);
      }
    },
    commonjsOptions: {
      transformMixedEsModules: true
    }
  },
  optimizeDeps: {
    include: [],
    exclude: ['@dfinity/agent', '@dfinity/auth-client', '@dfinity/candid', '@dfinity/principal', 'declarations'],
    esbuildOptions: {
      define: {
        global: "globalThis",
      },
      target: 'es2015',
      keepNames: true
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
    nodePolyfills({
      include: ['buffer', 'process', 'util'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
    environment("all", { prefix: "CANISTER_" }),
    environment("all", { prefix: "DFX_" }),
    {
      name: 'commonjs-externals',
      config(config) {
        config.define = config.define || {};
        config.define.global = 'globalThis';
      },
    },
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
      {
        find: "borc",
        replacement: fileURLToPath(new URL("../../node_modules/borc/src/index.js", import.meta.url)),
      },
      {
        find: "simple-cbor",
        replacement: fileURLToPath(new URL("../../node_modules/simple-cbor/src/index.js", import.meta.url)),
      },
    ],
    dedupe: ['@dfinity/agent'],
  },
});
