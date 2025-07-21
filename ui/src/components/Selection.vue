<script setup lang="ts">
import { isFairy, limit, pieces, price, updateCounter } from '@/helpers/points'
import { useGameStore } from '@/stores/game'
import { useShopStore } from '@/stores/game/useShopStore'
import { useGameSettings } from '@/stores/gameSettings'
import { computed, onMounted, onUnmounted, ref } from 'vue'

const gameStore = useGameStore()
const shopStore = useShopStore()
const settings = useGameSettings()

type Piece = {
  piece: string
  points: number
  max: number
  current: number
}

const pieceStyle = computed(() => {
  return function (piece: string) {
    let myColor = gameStore.playerColor()
    if (myColor == 'none') myColor = 'white'
    return `piece ${piece}-piece ${myColor}`
  }
})

function resize(_: UIEvent) {
  if (window.innerWidth >= 640) {
    if (selection.value == undefined) return
    const rect = selection.value.getBoundingClientRect()
    height.value = rect.width / 8
    return
  }
  height.value = window.innerWidth / 8
}

function wrapperSize() {
  return `--piece-size: ${height.value}px`
}

const allPieces = computed(() => {
  let all = pieces.filter((item) => {
    if (item == 'k') {
      return false
    } else if (!isFairy(gameStore.state.variant)) {
      if (item == 'a') return false
      else if (item == 'c') return false
      else if (item == 'g') return false
    }
    return true
  })
  return all
})

const counter = computed(() => {
  let t1 = price.slice()
  let t2 = limit.slice()
  updateCounter(gameStore.state.variant, t1, t2)

  return [t1, t2]
})

function isFairyPiece(p: Piece) {
  if (p.piece == 'a') return true
  else if (p.piece == 'c') return true
  else if (p.piece == 'g') return true
  return false
}
const height = ref(0)
const selection = ref<HTMLElement>()

onMounted(() => {
  if (selection.value == undefined) return

  resize({} as UIEvent)
  window.addEventListener('resize', resize)
})

onUnmounted(() => {
  window.removeEventListener('resize', resize)
})
</script>

<template>
  <div
    class="row-span-1 col-span-1 sm:col-start-1 sm:col-end-2 sm:row-start-2 sm:row-end-9 md:col-start-1 md:col-end-2 bg-linear-60 from-main-200 to-main-100 dark:from-main-800 dark:to-main-900 shadow shadow-main-300 dark:shadow-main-950 text-center font-sec"
  >
    <p class="underline underline-offset-4 dark:text-main-100 text-lg xl:text-xl mb-2">
      Credit: {{ shopStore.state.credit }} points
    </p>
    <div
      ref="selection"
      id="selection"
      :data-piece="settings.piece"
      class="cg-wrap !flex flex-col justify-between gap-y-3"
      :style="wrapperSize()"
    >
      <div
        v-for="(piece, index) in allPieces"
        class="flex flex-wrap text-2xl justify-around items-center font-main text-main-950 dark:text-main-100"
      >
        <div>
          <piece class="" :class="pieceStyle(piece)"></piece>
        </div>
        <div>{{ shopStore.state.pieceCounter[index + 1] }} / {{ counter[1][index + 1] }}</div>
        <div>{{ counter[0][index + 1] }} points</div>
        <button
          @click="shopStore.buy(piece, gameStore.playerColor())"
          class="cursor-pointer font-main-bold border border-main-800 dark:border-main-400 bg-main-300/30 dark:bg-main-800 shadow p-2"
        >
          +
        </button>
      </div>
    </div>
    <div class="p-5">
      <span
        @click="shopStore.confirm()"
        class="cursor-pointer border p-3 text-main-950 dark:text-main-100 font-sec md:text-xl hover:bg-main-300 dark:hover:bg-main-950 rounded-md"
        >Confirm</span
      >
    </div>
  </div>
</template>

<style>
@import '@/../public/board/chessground.css';

#selection piece {
  position: static;
  display: block;

  width: var(--piece-size);
  height: var(--piece-size);
}
</style>
