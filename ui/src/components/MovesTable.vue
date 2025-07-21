<script setup lang="ts">
import { computed, ref } from 'vue'
import FenMove from './FenMove.vue'
import FenButtons from './FenButtons.vue'
import { useUser } from '@/stores/user'
import { useGameStore } from '@/stores/game'
import { useWs } from '@/stores/ws'

let moves = ref(['a1', 'a2', 'a3', 'a4', 'rxa5'])
for (let i = 0; i < 10; i++) {
  moves.value.push(`a${i}`)
}

let ws = useWs()
let gameStore = useGameStore()

const live = computed(() => {
  return gameStore.state.status < 0 && !gameStore.state.players.includes('')
})

const history = computed(() => {
  let moves = gameStore.history().slice()
  if (gameStore.clientStage() == 1) {
    moves = moves.filter((item) => item.includes('@'))
  } else if (gameStore.clientStage() == 2) {
    moves = moves.filter((item) => {
      return item.split(' ').length == 6
    })
  }
  return moves
})
</script>

<template>
  <div
    class="row-span-1 md:flex-grow-1 md:shrink-1 md:basis-0 landscape:basis-[0%] landscape:row-end-9 landscape:col-start-2 landscape:col-end-3 sm:row-end-8 sm:col-start-2 sm:col-end-3 mt-2 flex flex-col h-[320px] sm:h-auto landscape:h-auto"
    :class="{
      ' landscape:row-start-4 sm:row-start-4 ': ws.reg && live,
      ' landscape:row-start-3 sm:row-start-3 ': !ws.reg || !live,
    }"
  >
    <FenButtons />
    <div
      class="grow-1 shrink-1 basis-[0%] sm:basis-0 shadow shadow-main-400 dark:shadow-main-800 overflow-y-scroll"
    >
      <ul id="movelist" class="flex flex-row flex-wrap overflow-y-scroll">
        <FenMove :value="value" :index="index" v-for="(value, index) in history"> </FenMove>
      </ul>
    </div>
  </div>
</template>
