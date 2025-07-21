<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import router from '@/router'
import { GET } from '@/helpers/fetch'
import * as v from 'valibot'
import { newTitle } from '@/helpers/backend'
import UserProfileGame from '../UserProfileGame.vue'
import { templateRef } from '@vueuse/core'
import init from 'shuuro-wasm'
import type { ShuuroGame, UserProfileGames } from '@/helpers/rust_types'
const sword = '"'
const route = useRoute()
let currentPage = ref(-1)
const games = ref([] as ShuuroGame[])
const name = ref('Player')
const userDate = ref('')
const scrollComponent = templateRef('container')
const wasmInit = ref(false)

const wasmGames = computed(() => {
  if (wasmInit.value == false) {
    return []
  }
  return games.value
})

function handleScroll() {
  let element = scrollComponent.value as unknown as HTMLDivElement
  if (element.getBoundingClientRect().bottom < window.innerHeight) {
    currentPage.value += 1
    newGames(currentPage.value)
  }
}

async function newGames(page: number) {
  let req = await GET(`/vue/@/${route.params.id}/${page}`)
  if (req.statusCode.value != 200) return
  let response: UserProfileGames = JSON.parse(req.data.value as string)
  if (page < 2 && response.player == undefined) {
    router.push('/')
    return
  } else if (response.games == undefined) {
    router.push('/')
    return
  }
  if (response.player != undefined) {
    name.value = response.player._id
    // @ts-ignore
    let date = new Date(Number(response.player.created_at['$date']['$numberLong']))
    userDate.value = date.toLocaleDateString()
  }
  games.value.push(...response.games)
}

onMounted(async () => {
  document.body.dataset['page'] = 'profile'
  if (route.params.id == undefined) router.push('/')
  // /vue/@/{username}/{page}

  newTitle(route.params.id as string)
  currentPage.value += 1
  await newGames(currentPage.value)
  window.addEventListener('scroll', handleScroll)
  await init()
  wasmInit.value = true
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
  watcher()
})

let watcher = watch(route, (newRoute, _oldRoute) => {
  let username = newRoute.params.username
  if (!username || newRoute.params.username == undefined) return
  games.value = []
  currentPage.value = 0
  newGames(0)
})
</script>

<template>
  <main
    class="col-start-3 col-end-4 grid grid-rows-[1rem_3vh_auto] text-main-950 dark:text-main-200"
  >
    <div class="row-span-1 col-span-1">&nbsp;</div>
    <div
      class="row-span-1 col-span-1 rounded-[50%] border-t-6 border-dashed border-main-300/70 dark:border-main-950/20"
      ref="firstItem"
    >
      &nbsp;
    </div>
    <div
      class="row-span-1 col-span-1 flex flex-col bg-linear-to-r to-75% lg:to-50% from-main-300 dark:from-main-800 to-main-200 dark:to-main-900 p-3"
    >
      <h1 class="text-xl font-main md:text-2xl lg:text-3xl xl:text-4xl">{{ name }}</h1>
      <h3
        class="row-span-1 col-span-1 flex flex-col items-end gap-3 py-3 text-sm border-b-2 border-main-400 dark:border-main-700 lg:text-2xl"
      >
        created at {{ userDate }}
      </h3>
    </div>
    <div class=""></div>
    <div
      ref="container"
      class="row-span-1 col-span-1 flex flex-col items-around lg:grid lg:grid-cols-[0.1fr_1fr_0.1fr] xl:grid-cols-[0.2fr_1.4fr_0.2fr] gap-5 justify-around bg-main-200/70 dark:bg-main-900 shadow-md shadow-main-300 dark:shadow-main-700 p-5"
    >
      <UserProfileGame :game="game" v-for="game in wasmGames" />
    </div>
  </main>
</template>
