<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import Clock from '../Clock.vue'
import ChessgroundView from '../ChessgroundView.vue'
import FinishGameButtons from '../FinishGameButtons.vue'
import MovesTable from '../MovesTable.vue'
import PlacementPocket from '../PlacementPocket.vue'
import Selection from '../Selection.vue'
import { useMaxWidthStore } from '@/stores/maxWidth'
import GameInfo from '../GameInfo.vue'
import { ntw } from '@/not-tailwind'
import { defineAsyncComponent } from 'vue'
import { useWs } from '@/stores/ws'
import { useRoute } from 'vue-router'
import router from '@/router'
import { useGameStore } from '@/stores/game'
import { GET } from '@/helpers/fetch'
import { MessageType, type ShuuroGame } from '@/helpers/rust_types'

const route = useRoute()

const MainModal = defineAsyncComponent({
  loader: () => import('../MainModal.vue'),
  timeout: 300,
})

const RedirectToast = defineAsyncComponent({
  loader: () => import('../RedirectToast.vue'),
})

const darkTheme = `var(${ntw.get('--color-main-900')})`
const lightTheme = `var(${ntw.get('--color-main-100')})`

const width = ref(0)
const firstItem = ref<HTMLElement>()

const store = useMaxWidthStore()

function resize(_: UIEvent) {
  width.value = window.innerWidth
  store.resize()
  /*if(window.innerHeight >= 1024) {
    
  }
  */
}

function mediumScreen() {
  return width.value >= 768
}
const ws = useWs()
const game = useGameStore()
game.listen()

let gameId = ''

function gameProps(): ShuuroGame | undefined | false {
  // @ts-ignore
  let gameProps = window.gameProps
  if (gameProps == undefined) {
    return
  } else if (gameProps._id == undefined) {
    return false
  }
  try {
    // @ts-ignore
    return window.gameProps
  } catch (e) {
    return undefined
  }
}

onMounted(async () => {
  document.body.dataset['page'] = 'game'
  store.firstItem = firstItem.value
  resize({} as UIEvent)
  window.addEventListener('resize', resize)
  if (route.params.id == undefined) router.push('/')
  gameId = route.params.id as string
  let state = gameProps()
  if (state == undefined || state == false) {
    let state = await GET(`/vue/game/${gameId}`)
    let newstate: ShuuroGame = JSON.parse(state.data.value as string)
    if (newstate._id == undefined) router.push('/')
    game.fromServer(newstate)
    // @ts-ignore
    delete window.gameProps
  } else {
    game.fromServer(Object.assign({}, state))
    // @ts-ignore
    delete window.gameProps
  }
})

onUnmounted(() => {
  if (route.params.id == undefined && route.fullPath.includes('game/')) router.push('/')
  ws.SEND({ t: MessageType.ChangeRoom, d: '' })
  game.stopListening()
  window.removeEventListener('resize', resize)
  game.unMounted()
})
</script>

<template>
  <main
    class="col-start-3 col-end-4 grid grid-rows-[repeat(3,auto)] sm:grid-cols-[minmax(55vmin,calc(100vh-calc(var(--header-height)_+_1.75rem_+_var(--empty-item-height,_0px))))_minmax(200px,var(--lg-a,270px))] landscape:grid-rows-[5vh,repeat(10,auto)] sm:grid-rows-[3vh,_repeat(10,auto)] lg:grid-cols-[minmax(65vmin,calc(55vh-calc(var(--header-height,0px)_+_1.75rem_+_var(--empty-item-height,0px)_+_var(--placement-height,0px))))_minmax(240px,var(--lg-a,300px))] gap-y-3 landscape:gap-y-2 landscape:gap-x-[var(--lg-gap,0.75rem)] sm:gap-x-3 md:gap-8 lg:[--lg-gap:1rem] animate-[main-layout_0.5s]"
    :class="{
      ' landscape:grid-cols-[minmax(55vmin,calc((88vh-calc(var(--header-height)_+_1.75rem_+_calc(var(--placement-height,0px)/2)))*_var(--board-scale,1)))_minmax(190px,var(--lg-a,270px))] lg:grid-cols-[minmax(55vmin,calc(88vh-calc(var(--header-height)_+_1.75rem_+_calc(var(--placement-height,0px)/2))))_minmax(190px,var(--lg-a,270px))] lg:[--lg-a:320px] xl:[--lg-a:370px] ':
        game.clientStage() == 1,
      ' landscape:grid-cols-[minmax(60vmin,calc((100vh-calc(var(--header-height)_+_1.75rem_+_var(--empty-item-height)))*_var(--board-scale,1)))_minmax(190px,var(--lg-a,_230px))] lg:grid-cols-[minmax(60vmin,calc(100vh-calc(var(--header-height)_+_1.75rem_+_var(--empty-item-height))))_minmax(190px,var(--lg-a,_230px))] lg:[--lg-a:350px] xl:[--lg-a:370px] ':
        game.clientStage() >= 2,

      '  lg:grid-rows-[15vh_auto_auto] lg:[--lg-a:350px] ': game.clientStage() == 0,
    }"
  >
    <div
      ref="firstItem"
      v-if="game.clientStage() == 0"
      class="hidden sm:block sm:row-span-1 sm:col-span-2 lg:col-span-b lg:row-start-1 lg:row-end-2 lg:block lg:invisible"
    >
      &nbsp;
    </div>

    <Clock v-if="mediumScreen() == false" :stage="game.clientStage()" />
    <ChessgroundView v-if="game.clientStage() == 1 || game.clientStage() == 2" />
    <Selection v-if="game.clientStage() == 0" />
    <PlacementPocket v-if="game.clientStage() == 1" />
    <FinishGameButtons v-if="!mediumScreen()" />
    <MovesTable v-if="!mediumScreen()" />
    <GameInfo v-if="!mediumScreen()" />

    <div
      v-if="mediumScreen()"
      id="right-side"
      class="md:col-start-2 md:col-end-3 row-start-2 row-end-9 md:flex md:flex-col gap-2 lg:gap-y-5"
    >
      <Clock :stage="game.clientStage()" />
      <FinishGameButtons />
      <MovesTable />
      <GameInfo />
    </div>
    <MainModal />
    <Teleport to="body">
      <RedirectToast v-if="ws.redirectToastOpen" />
    </Teleport>
  </main>
</template>

<style scoped>
@keyframes main-layout {
  0% {
    background-color: v-bind(darkTheme);
  }
  100% {
    background-color: transparent;
  }
}

@media (prefers-color-scheme-dark) {
  @keyframes main-layout {
    0% {
      background-color: v-bind(lightTheme);
    }
    100% {
      background-color: transparent;
    }
  }
}
</style>
