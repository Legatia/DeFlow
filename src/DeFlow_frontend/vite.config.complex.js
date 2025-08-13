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
    target: 'es2020', // Support BigInt for @dfinity packages
    rollupOptions: {
      external: [],
      output: {
        format: 'iife',
        inlineDynamicImports: true,
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM'
        }
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
      transformMixedEsModules: true,
      include: ['borc', '@nfid/identitykit'],
      exclude: ['react', 'react-dom']
    }
  },
  optimizeDeps: {
    include: ['react', 'react-dom', 'react/jsx-runtime', 'borc', '@nfid/identitykit/react'],
    exclude: ['@dfinity/agent', '@dfinity/auth-client', '@dfinity/candid', '@dfinity/principal', 'declarations'],
    esbuildOptions: {
      define: {
        global: "globalThis",
      },
      target: 'es2020', // Support BigInt for @dfinity packages
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
      include: ['buffer', 'process', 'util', 'stream'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
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
    dedupe: ['@dfinity/agent', 'react', 'react-dom'],
  },
  define: {
    // Fix for BORC module issues
    'process.env.NODE_DEBUG': 'false',
    global: 'globalThis',
  },
  ssr: {
    noExternal: ['@nfid/identitykit']
  },
});
