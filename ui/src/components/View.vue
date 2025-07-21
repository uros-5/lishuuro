<script setup lang="ts">
import { useMaxWidthStore } from '@/stores/maxWidth'
import { onMounted, ref } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import { useWs } from '@/stores/ws'

const route = useRoute()
const transitionEnd = ref(false)
const root = ref()
const ws = useWs()

onMounted(() => {
  if (root == undefined) return
  const store = useMaxWidthStore()
  store.header = root.value
})
</script>

<template>
  <header
    ref="root"
    class="flex flex-row justify-center text-main-950 dark:text-main-100 font-main"
  >
    <Transition name="header">
      <RouterLink v-if="route.name != 'home'" class="text-2xl lg:text-3xl mt-3" to="/"
        >lishuuro.org</RouterLink
      >
    </Transition>
  </header>

  <div
    id="main-wrap"
    class="grid grid-cols-[0px_0.5fr_minmax(auto,var(--main-max-width))_0.5fr_0px] md:auto-rows-auto gap-y-[50vh]"
    :class="{
      ' sm:grid-cols-[1vw_0.5fr_minmax(auto,auto)_0.5fr_1vw] ': route.name == 'game',
    }"
  >
    <RouterView v-slot="{ Component }">
      <transition v-if="route.name != 'game'" name="fast-fade">
        <component :is="Component" />
      </transition>

      <component v-else :is="Component" />
    </RouterView>
  </div>

  <!-- <div -->
  <!-- id="main-wrap" -->
  <!-- v-if="router.name == 'game'" -->
  <!-- class="grid grid-cols-[0px_1fr_minmax(auto,calc(auto))_1fr_0px]" -->
  <!-- > -->
  <!-- <RouterView /> -->
  <!-- </div> -->
  <footer>
    <div
      id="hamburger-icon"
      v-if="route.name != 'home'"
      class="hidden fixed justify-items-center gap-1.5 z-6 -bottom-0.5 right-0 p-3 md:hidden cursor-pointer"
    >
      <span v-for="_ in 3" class="h-1 w-8 rounded-full bg-black dark:bg-white"></span>
    </div>
    <!-- hidden part -->
    <div class="hidden">
      <div class="top-6 left-3 dark:bg-gray-200 dark:bg-white-200"></div>
      <div class="blur-2xl"></div>
    </div>
  </footer>
</template>

<style>
.fast-fade-enter-active {
  transition:
    opacity 1.7s ease,
    transform 0.7s ease;
}

.fast-fade-leave-active {
  transition: opacity 0.5s ease-out;
}

.fast-fade-enter-from {
  opacity: 0;
}

.fast-fade-leave-to {
  opacity: 0;
}

.header-enter-active,
.header-leave-active {
  transition: font-size 0.3s ease;
}

.header-enter-from,
.header-leave-to {
  font-size: 0px;
}
</style>
