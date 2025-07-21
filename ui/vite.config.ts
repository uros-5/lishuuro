import { fileURLToPath, URL } from 'node:url'
import vueDevTools from "vite-plugin-vue-devtools"

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import Sonda from "sonda/vite";

const CUSTOM_ELEMENTS = ["info-date", "selection", "square", "piece", "cg-board", "cg-container", "piece", "i-side.online", "player-title", "rating", "player", "round-player0", "san", "eval", "i-side.online", "piece", "i-side"];
// https://vite.dev/config/
export default defineConfig({

  assetsInclude: ['**/*.ogg'],

  optimizeDeps: { exclude: ["shuuro-wasm"] },
  

  plugins: [
    vue(
      {
        template: {
          compilerOptions: {
            isCustomElement: (tag) => CUSTOM_ELEMENTS.includes(tag)
          }
        }
      }
    ), tailwindcss(), vueDevTools(), Sonda()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  build: {
    cssCodeSplit: true,
    rollupOptions: {
      input: {
        main: "src/components/index.html",
        tailwind: "src/components/tailwind.html",
      },
    },
    sourcemap: true,
  }
})


