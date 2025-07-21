<script setup lang="ts">
import { FenBtn } from '@/helpers/fen'
import { useGameStore } from '@/stores/game'

const gameStore = useGameStore()
</script>

<template>
  <div
    class="text-xl flex flex-row justify-around bg-gradient-to-b from-main-200 to-main-400 dark:from-main-700 dark:to-main-950 text-main-800 dark:text-main-200 hover:cursor-pointer"
  >
    <button
      @click="gameStore.findFen(FenBtn.First)"
      data-act="first"
      data-icon=""
      class="grow-1 hover:scale-80 transition-transform shrink-1 basis-[20%] text-center md:text-xl before:md:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 before:content-[attr(data-icon)] hover:cursor-pointer"
    ></button>
    <button
      @click="gameStore.findFen(FenBtn.Previous)"
      data-act="prev"
      data-icon=""
      class="grow-1 hover:scale-80 transition-transform shrink-1 basis-[20%] text-center md:text-xl before:md:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 before:content-[attr(data-icon)] hover:cursor-pointer"
    ></button>
    <button
      @click="gameStore.findFen(FenBtn.Next)"
      data-act="next"
      data-icon=""
      class="grow-1 hover:scale-80 transition-transform shrink-1 basis-[20%] text-center md:text-xl before:md:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 before:content-[attr(data-icon)] hover:cursor-pointer"
    ></button>
    <button
      @click="gameStore.findFen(FenBtn.Last)"
      data-act="last"
      data-icon=""
      class="grow-1 hover:scale-80 transition-transform shrink-1 basis-[20%] text-center md:text-xl before:md:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 before:content-[attr(data-icon)] hover:cursor-pointer"
    ></button>
  </div>
</template>
