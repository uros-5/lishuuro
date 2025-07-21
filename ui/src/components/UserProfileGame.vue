<script setup lang="ts">
import { useRoute } from 'vue-router'
import ProfileChessground from './ProfileChessground.vue'
import { computed, ref } from 'vue'
import { timeago } from '@/helpers/timeago'
import { useWs } from '@/stores/ws'
import type { ShuuroGame } from '@/helpers/rust_types'
const sword = '"'
const ws = useWs()
let props = defineProps<{ game: ShuuroGame }>()

const min = computed(() => {
  return props.game.min / 60000
})

const incr = computed(() => {
  return props.game.incr / 1000
})

const playedAt = computed(() => {
  let converted = new Date(props.game.tc.last_click).toString()
  return timeago(converted)
})

function result() {
  let player = 'Both'
  if (props.game.result != 2) {
    player = props.game.players[props.game.result]
  }
  player = ws.username == player ? 'You' : player

  if (props.game.status == 1) {
    return `Checkmate by ${player}.`
  } else if (props.game.status == 3) {
    return 'Stalemate.'
  } else if (props.game.status == 4) {
    return 'Draw by repetition.'
  } else if (props.game.status == 5) {
    return 'Draw by agreement.'
  } else if (props.game.status == 6) {
    return 'Draw by material.'
  } else if (props.game.status == 7) {
    return `${player} resigned.`
  } else if (props.game.status == 8) {
    if (props.game.result == 2) {
      player = 'Both players'
    }
    return `${player} lost on time.`
  } else if (props.game.status == 9) {
    return `${player} lost on first move by checkmate.`
  }
}
</script>

<template>
  <RouterLink
    :to="`/game/${props.game._id}`"
    class="grid grid-cols-2 lg:col-start-2 lg:col-end-3 sm:gap-3 sm:grid-cols-[auto_1fr] lg:grid-cols-[auto_1fr] sm:grid-rows-[300px_auto] lg:grid-rows-[256px_auto] inset-ring-teal-50 shadow shadow-main-500 dark:shadow-main-700 font-sec bg-linear-to-r from-main-400/50 dark:from-main-700 to-60% to-main-200/80 dark:to-main-800/70 hover:cursor-pointer hover:bg-main-400/90 hover:dark:bg-main-700 hover:scale-101 transition-[background_transform] duration-400 rounded sm:p-3 lg:p-6"
  >
    <ProfileChessground :game="props.game" />
    <div
      class="row-span-1 col-span-2 sm:row-span-0 sm:col-span-1 grid grid-cols-3 grid-rows-[0.8fr_0.2fr_0.2fr_0.2fr] sm:grid-rows-[0.8fr_0.2fr_0.2fr_0.2fr] lg:grid-rows-[0.6fr_0.2fr_0.2fr_0.2fr] text-sm sm:text-xl md:text-2xl"
    >
      <div
        class="flex row-span-1 col-span-3 flex-col lg:flex-row gap-3 lg:gap-5 lg:justify-center items-center py-5"
      >
        <h1 class="text-main-950 dark:text-main-50">{{ props.game.players[1] }}</h1>
        <div
          class="font-lishuuro text-main-800 dark:text-main-200 text-2xl sm:text-3xl pl-1 pt-2 pr-0 pb-2 before:text-3xl before:align-middle before:content-[attr(data-icon)]"
          :data-icon="sword"
        ></div>
        <h1 class="text-main-950 dark:text-main-50">{{ props.game.players[0] }}</h1>
      </div>

      <div
        class="row-start-2 row-end-3 col-start-1 col-end-4 bg-main-400/20 dark:bg-main-700/20 text-center"
      >
        <p class="text-2xl font-sec border-t-1 border-main-400 dark:border-main-600">
          {{ result() }}
        </p>
      </div>

      <div
        class="row-start-3 row-end-4 col-start-1 col-end-3 bg-linear-to-r to-80% from-main-400/80 dark:from-main-700"
      >
        {{ min }}m + {{ incr }}s
      </div>
      <div
        class="row-start-4 row-end-5 sm:row-start-3 sm:row-end-4 bg-linear-to-l to-80% from-main-400/80 dark:from-main-700 col-start-2 col-end-4 text-right"
      >
        {{ playedAt }}
      </div>
    </div>
  </RouterLink>
</template>
