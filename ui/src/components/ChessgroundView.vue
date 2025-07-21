<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import { Geometry, type MouchEvent } from 'chessground12/types'
import { useMaxWidthStore } from '@/stores/maxWidth'
import { useGameSettings } from '@/stores/gameSettings'
import { templateRef } from '@vueuse/core'
import { useGameStore } from '@/stores/game'
import { useCgStore } from '@/stores/game/useCgStore'
import { useWasmStore } from '@/stores/game/useWasmStore'
import { cgInfo } from '@/stores/tvStore'
import { FenBtn } from '@/helpers/fen'

const chessground = templateRef('chessground')
const gameStore = useGameStore()
const cgStore = useCgStore()
const wasmStore = useWasmStore()
const store = useMaxWidthStore()

const geometry = computed(() => {
  return cgInfo(gameStore.state.variant)[0]
})
const settings = useGameSettings()
//const geometry = ref(Geometry.dim12x12)

function dataSize() {
  if (geometry.value == Geometry.dim8x8) {
    return 8
  } else if (geometry.value == Geometry.dim12x12) {
    return 12
  } else if (geometry.value == Geometry.dim6x6) {
    return 6
  }
}

function eventPosition(e: MouchEvent): [number, number] | undefined {
  if (e.clientX || e.clientX === 0) return [e.clientX, e.clientY!]
  if (e.targetTouches?.[0]) return [e.targetTouches[0].clientX, e.targetTouches[0].clientY]
  return
}

onMounted(async () => {
  cgStore.state.element = chessground.value
  gameStore.newClientStage(2)
  await wasmStore.state.init
  await gameStore.user.player.loaded
  store.resize()
  cgStore.fightCg()
})

function wheel(event: WheelEvent) {
  const btn = event.deltaY >= 0 ? FenBtn.Next : FenBtn.Previous
  gameStore.findFen(btn)
}
</script>

<template>
  <div
    class="relative row-span-1 col-start-1 col-end-2 landscape:row-start-2 landscape:row-end-9 landscape:mt-3 landscape:col-start-1 landscape:col-end-2 sm:row-start-2 sm:row-end-9 sm:mt-3 sm:col-start-1 sm:col-end-2 md:col-start-1 md:col-end-2 block h-0 pb-[100%] w-full"
  >
    <div
      ref="chessground"
      id="chessground12"
      :data-size="dataSize()"
      :data-board="settings.board"
      :data-piece="settings.piece"
      class="cg-wrap absolute w-full h-full"
      @wheel.prevent="wheel"
    ></div>
  </div>
</template>
<style>
@import '@/../public/board/chessground.css';
</style>
