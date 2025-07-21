<script setup lang="ts">
import { MessageType } from '@/helpers/rust_types'
import { useGameStore } from '@/stores/game'

import { useWs } from '@/stores/ws'

let ws = useWs()
let gameStore = useGameStore()

function draw() {
  let d = confirm('Are you sure you want to draw?')
  if (d) {
    ws.SEND({ t: MessageType.Draw, d: null })
  }
}

function accept() {
  ws.SEND({ t: MessageType.Draw, d: null })
  gameStore.user.offeredDraw = false
}

function deny() {
  gameStore.user.offeredDraw = false
}

function resign() {
  let d = confirm('Are you sure you want to resign?')
  if (d) {
    ws.SEND({ t: MessageType.Resign, d: null })
  }
}

function result() {
  let player = 'Both'
  if (gameStore.state.result != 2) {
    player = gameStore.state.players[gameStore.state.result]
  }
  player = ws.username == player ? 'You' : player

  if (gameStore.state.status == 1) {
    return `Checkmate by ${player}.`
  } else if (gameStore.state.status == 3) {
    return 'Stalemate.'
  } else if (gameStore.state.status == 4) {
    return 'Draw by repetition.'
  } else if (gameStore.state.status == 5) {
    return 'Draw by agreement.'
  } else if (gameStore.state.status == 6) {
    return 'Draw by material.'
  } else if (gameStore.state.status == 7) {
    return `${player} resigned.`
  } else if (gameStore.state.status == 8) {
    if (gameStore.state.result == 2) {
      player = 'Both players'
    }
    return `${player} lost on time.`
  } else if (gameStore.state.status == 9) {
    return `${player} lost on first move by checkmate.`
  }
}
</script>

<template>
  <div
    v-if="gameStore.player().isPlayer && gameStore.state.status < 0"
    class="col-start-1 col-end-2 row-span-1 landscape:row-start-3 landscape:row-end-4 landscape:col-start-2 landscape:col-end-3 landscape:h-min sm:row-start-3 sm:row-end-4 sm:col-start-2 sm:col-end-3 sm:h-min flex flex-wrap justify-around text-main-950 dark:text-main-50 font-lichess p-3 gap-3"
  >
    <button
      @click="draw()"
      class="border shadow-main-500 cursor-pointer basis-[40%] rounded before:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)]"
      data-icon=""
    ></button>
    <button
      @click="resign()"
      class="border shadow-main-500 cursor-pointer basis-[40%] rounded before:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)]"
      data-icon=""
    ></button>

    <div
      class="basis-[100%] grid grid-cols-[0.4fr_0.2fr_0.4fr] grid-rows-2 gap-y-3"
      v-if="gameStore.user.offeredDraw"
    >
      <p class="col-start-1 col-end-4 text-xl cursor-pointer text-center">Do you want a draw?</p>

      <button
        @click="accept()"
        class="col-start-1 col-end-2 border shadow-main-500 cursor-pointer rounded before:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)]"
        data-icon=""
      ></button>

      <button
        @click="deny()"
        class="col-start-3 col-end-4 border shadow-main-500 cursor-pointer basis-[45%] rounded before:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 bg-gradient-to-b hover:from-main-400 hover:dark:from-main-950 dark:from-main-800 from-main-200 dark:to-main-900 to-main-100 before:content-[attr(data-icon)]"
        data-icon=""
      ></button>
    </div>
  </div>
  <div
    v-else-if="gameStore.state.status > 0"
    class="col-start-1 col-end-2 row-span-1 landscape:row-start-3 landscape:row-end-4 landscape:col-start-2 landscape:col-end-3 landscape:h-min sm:row-start-3 sm:row-end-4 sm:col-start-2 sm:col-end-3 sm:h-min flex flex-wrap justify-around text-main-950 dark:text-main-50 font-lichess p-3 gap-3"
  >
    <p class="font-main">{{ result() }}</p>
  </div>
</template>
