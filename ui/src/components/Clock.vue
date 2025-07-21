<script setup lang="ts">
import { useGameStore } from '@/stores/game'
import { useClockStore } from '@/stores/game/useClockStore'
import { useMaxWidthStore } from '@/stores/maxWidth'

import { computed, onMounted, onUnmounted, ref } from 'vue'

const props = defineProps<{ stage: number }>()

const game = useGameStore()
const clockStore = useClockStore()

let players = computed(() => {
  let i = game.user.player.player
  if (i == 1) return game.state.players
  else return game.state.players.slice().reverse()
})

let clocks = computed(() => {
  let i = game.user.player.player
  if (i == 1) return clockStore.state.clocks
  else return clockStore.state.clocks.slice().reverse()
})

const root = ref()

onMounted(() => {
  if (root == undefined) return
  const store = useMaxWidthStore()
  store.clock = root.value
})
</script>

<template>
  <div
    ref="root"
    class="row-span-1 col-start-1 col-end-2 landscape:row-start-2 landscape:row-end-3 landscape:col-start-2 landscape:col-end-3 sm:row-start-2 sm:row-end-3 sm:col-start-2 sm:col-end-3 grid grid-cols-5 grid-rows-[repeat(5,_auto)] bg-linear-240 dark:bg-linear-240 from-main-200 dark:from-main-950 to-main-300 dark:to-main-900 shadow-sm shadow-main-500 dark:shadow-main-800 mt-2 md:h-min landscape:h-min"
  >
    <div
      class="col-span-3 row-span-1 flex flex-col xl:text-lg text-main-950 dark:text-main-100 font-sec p-1"
    >
      <RouterLink v-for="i in players" :to="`/@/${i}`">{{ i == '' ? 'waiting..' : i }}</RouterLink>
    </div>
    <div
      class="col-span-2 row-span-1 flex flex-col border-l-2 border-main-800 text-center font-[roboto]"
    >
      <div class="text-xl lg:text-3xl xl:text-4xl text-main-950 dark:text-main-200">
        {{ clocks[0].currentMin }}:{{ clocks[0].currentSec }}
      </div>
      <div class="border-b-2 text-main-950 dark:text-main-200"></div>
      <div class="text-xl lg:text-3xl xl:text-4xl text-main-950 dark:text-main-200">
        {{ clocks[1].currentMin }}:{{ clocks[1].currentSec }}
      </div>
    </div>
  </div>
</template>
