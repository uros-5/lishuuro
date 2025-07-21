<script setup lang="ts">
import { useGameSettings } from '@/stores/gameSettings'
import { cgInfo, tvCg, useTvStore, type TvGame } from '@/stores/tvStore'
import { templateRef } from '@vueuse/core'
import { Geometry } from 'chessground12/types'
import { computed, onMounted } from 'vue'

const props = defineProps<{ game: TvGame }>()

const settings = useGameSettings()
const element = templateRef('element')

const geometry = computed(() => {
  return cgInfo(props.game.variant)[0]
})

function dataSize() {
  if (geometry.value == Geometry.dim8x8) {
    return 8
  } else if (geometry.value == Geometry.dim12x12) {
    return 12
  } else if (geometry.value == Geometry.dim6x6) {
    return 6
  }
}

onMounted(async () => {
  if (element.value == undefined) return
  let cg = tvCg(element.value, props.game)
  const tvStore = useTvStore()
  tvStore.updateCg(props.game.id, cg)
})
</script>

<template>
  <RouterLink
    :to="`/game/${props.game.id}`"
    class="flex flex-col justify-center items-center gap-2 m-1 p-2 bg-main-100 dark:bg-main-800 shadow shadow-main-300 dark:shadow-main-800 rounded"
  >
    <div class="text-left pt-1 sm:text-xl md:text-2xl">{{ props.game.players[1] }}</div>
    <div class="standard">
      <div>
        <div
          ref="element"
          :data-board="settings.board"
          :data-piece="settings.piece"
          :data-size="dataSize()"
          class="mini w-auto pb-[100%]"
        />
      </div>
    </div>
    <div class="text-left pt-1 sm:text-xl md:text-2xl">{{ props.game.players[0] }}</div>
  </RouterLink>
</template>

<style>
@import '@/../public/board/chessground.css';
</style>
