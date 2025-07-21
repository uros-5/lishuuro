<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useTvStore } from '@/stores/tvStore'
import { useWs } from '@/stores/ws'
import TvGame from '../TvGame.vue'
import { newTitle } from '@/helpers/backend'
import { MessageType } from '@/helpers/rust_types'

const ws = useWs()
const tvStore = useTvStore()
tvStore.listen()

onMounted(async () => {
  await tvStore.init()
  document.body.dataset['page'] = 'tv'
  ws.SEND({ t: MessageType.ChangeRoom, d: 'tv' })
  ws.SEND({ t: MessageType.GetTv, d: null })
  newTitle('TV')
})

onUnmounted(async () => {
  tvStore.stopListening()
})
</script>

<template>
  <main class="col-start-3 col-end-4 grid grid-rows-[3vh_auto] text-main-950 dark:text-main-200">
    <div class="row-span-1 col-span-1">&nbsp;</div>

    <div class="row-span-1 col-span-1 p-3">Watch multiple games</div>

    <TransitionGroup
      name="tv-games"
      tag="div"
      class="row-span-1 col-span-1 grid grid-cols-[repeat(auto-fill,minmax(24rem,1fr))] justify-center overflow-x-hidden gap-1"
    >
      <TvGame :game="game" v-for="game in tvStore.games" />
    </TransitionGroup>

    <div class=""></div>
  </main>
</template>

<style>
.tv-games-leave-active {
  transition:
    filter 0.5s ease,
    opacity 0.7s ease;
}

.tv-games-enter-from,
.tv-games-leave-to {
  opacity: 0;
  filter: blur(4px);
}
</style>
