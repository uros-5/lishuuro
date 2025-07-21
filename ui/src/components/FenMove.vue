<script setup lang="ts">
import { FenBtn } from '@/helpers/fen'
import { useGameStore } from '@/stores/game'

const props = defineProps<{ value: string; index: number }>()
const gameStore = useGameStore()

function bigIndex() {
  return props.index / 2 + 1 >= 100
}

function fen() {
  if (gameStore.clientStage() == 0) {
    return props.value
  }

  return props.value.split(' ')[4]
}

function index() {
  return gameStore.clientStage() == 0 ? props.index : props.index + 1
}

function pickMove() {
  if (gameStore.clientStage() == 0) return
  gameStore.user.historyIndex = props.index + 2
  gameStore.findFen(FenBtn.Previous)
}

// last:bg-main-100 last:dark:bg-main-600
</script>

<template>
  <li
    v-if="props.index % 2 == 0"
    class="basis-[20%] text-xl text-main-950 dark:text-main-100 bg-main-300 dark:bg-main-900 border-r-2 border-r-main-700 text-center"
  >
    {{ props.index / 2 + 1 }}
  </li>
  <li
    @click="pickMove()"
    :data-fen="props.value"
    class="basis-[40%] text-xl text-main-950 dark:text-main-100 px-2 hover:bg-main-400 dark:hover:bg-main-950 cursor-pointer"
    :class="{
      ' border-r-2 ': props.index % 2 == 0,
      ' bg-main-100 dark:bg-main-600 ': gameStore.user.historyIndex == index(),
      ' bg-main-200 dark:bg-main-700 ': gameStore.user.historyIndex != index(),
    }"
  >
    {{ fen() }}
  </li>
</template>
