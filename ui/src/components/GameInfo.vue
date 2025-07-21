<script setup lang="ts">
import { timeago } from '@/helpers/timeago'
import { useGameStore } from '@/stores/game'
import { useModal } from '@/stores/modal'
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

const modal = useModal()

const game = useGameStore()

let players = computed(() => {
  let i = game.user.player.player
  if (i == 1) return game.state.players
  else return game.state.players.slice().reverse()
})

function dataIcon(name: string) {
  if (!game.user.player.isPlayer) return ''
  if (name == '') return ''
  let myColor = game.playerColor(name)
  if (myColor == 'white') {
    return 'b'
  } else if (myColor == 'black') {
    return 'c'
  }
  return 'z'
}

function gameDate(): string {
  if (game.state.status < 0) {
    return 'Playing right now'
  } else {
    let converted = new Date(game.state.tc.last_click).toString()
    let a = timeago(converted)
    return a
  }
}
</script>

<template>
  <div
    class="row-span-1 flex flex-col gap-3 text-main-900 dark:text-main-200 sm:col-start-2 sm:col-end-3 sm:row-start-8 sm:row-end-9 bg-linear-to-br to-70% from-main-100 dark:from-main-800 to-main-200 dark:to-main-900 shadow shadow-main-100 dark:shadow-main-800 font-sec p-3"
  >
    <div class="flex justify-around border-b-1 text-2xl">
      <p
        data-icon="M"
        class="before:content-[attr(data-icon)] before:font-lishuuro before:text-3xl"
      ></p>
      <p>{{ game.state.min }}m + {{ game.state.incr }}</p>
    </div>

    <div v-for="name in players" class="pr-2 flex gap-2">
      <i-side
        :data-icon="dataIcon(name)"
        class="before:content-[attr(data-icon)] before:font-lishuuro"
      ></i-side>
      <player>
        <RouterLink :to="`/@/${name}`" class="pr-3">{{ name }}</RouterLink>
      </player>
    </div>
    <div class="flex justify-around">
      <button
        @click="modal.toggle('game-settings')"
        data-icon=""
        class="before:md:text-xl before:align-middle font-lichess before:font-normal before:normal-case before:opacity-90 before:content-[attr(data-icon)] hover:cursor-pointer transition-transform duration-150 hover:scale-110"
      ></button>
      <p class="">{{ gameDate() }}</p>
    </div>
  </div>
</template>

<style>
.icon-white:before {
  content: '\63';
}
</style>
