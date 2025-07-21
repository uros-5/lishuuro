<script setup lang="ts">
import { useGameStore } from '@/stores/game'
import { useCgStore } from '@/stores/game/useCgStore'
import { useWasmStore } from '@/stores/game/useWasmStore'
import { useGameSettings } from '@/stores/gameSettings'
import { useMaxWidthStore } from '@/stores/maxWidth'
import { templateRef } from '@vueuse/core'
import { onMounted, onUnmounted, ref } from 'vue'

const root = templateRef('root')
const top = templateRef('top')
const bot = templateRef('bot')

const store = useMaxWidthStore()
const gameStore = useGameStore()
const cgStore = useCgStore()
const wasmStore = useWasmStore()
const settings = useGameSettings()

let mutationCount = 0
const observer = new MutationObserver((mutationsList) => {
  for (const mutation of mutationsList) {
    if (mutation.type == 'childList') {
      if (mutationCount <= 10) {
        store.resize()
        mutationCount += 1
        if (mutationCount == 10) {
          observer.disconnect()
        }
      }
    }
  }
})

onMounted(async () => {
  gameStore.newClientStage(1)
  if (!root) return
  store.placement = root.value

  observer.observe(root.value!, {
    childList: true,
    subtree: true,
  })
  cgStore.state.top = top.value
  cgStore.state.bot = bot.value
  await wasmStore.state.init
  await gameStore.user.player.loaded
  store.resize()
  cgStore.deployCg()
})

onUnmounted(() => {
  observer.disconnect()
})
</script>

<template>
  <div
    ref="root"
    :data-piece="settings.piece"
    class="row-span-1 col-span-1 md:row-start-9 md:row-end-10 landscape:row-start-9 landscape:row-end-10 landscape:col-start-1 landscape:col-end-2 sm:row-start-9 sm:row-end-10 sm:col-start-1 sm:col-end-2 md:col-start-1 md:col-end-2 flex flex-col cg-wrap pocket"
  >
    <div ref="top" id="black-pocket"></div>
    <div ref="bot" id="white-pocket"></div>
  </div>
</template>
