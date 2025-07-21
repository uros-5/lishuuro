<script setup lang="ts">
import { backend, frontend } from '@/helpers/backend'
import router from '@/router'
import { useGameStore } from '@/stores/game'
import { useWs } from '@/stores/ws'
import {
  ToastAction,
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastTitle,
  ToastViewport,
  ToastPortal,
} from 'reka-ui'

const store = useGameStore()
const ws = useWs()

function copyUrl() {
  try {
    navigator.clipboard.writeText(frontend() + '/game/' + store.state._id)
  } catch {}
}
</script>

<template>
  <ToastProvider>
    <ToastRoot
      v-model:open="ws.redirectToastOpen"
      class="bg-white toast-root-grid rounded-lg shadow-sm border p-[15px] grid grid-cols-[auto_max-content] gap-x-[15px] items-center data-[state=open]:animate-slideIn data-[state=closed]:animate-hide data-[swipe=move]:translate-x-[var(--reka-toast-swipe-move-x)] data-[swipe=cancel]:translate-x-0 data-[swipe=cancel]:transition-[transform_200ms_ease-out] data-[swipe=end]:animate-swipeOut"
    >
      <ToastTitle class="[grid-area:_title] mb-[5px] font-medium text-slate12 text-sm">
        Game started
      </ToastTitle>
      <ToastDescription as-child>
        <p class="[grid-area:_description] m-0 text-slate11 text-xs leading-[1.3]">Live</p>
        <!-- <p class="!select-text">{{ frontend() }}game/{{ store.state._id }}</p> -->
      </ToastDescription>
      <ToastAction class="[grid-area:_action]" as-child alt-text="Copy game url">
        <button
          @click="copyUrl"
          class="inline-flex items-center justify-center rounded-md font-medium text-xs px-[10px] leading-[25px] h-[25px] bg-green2 text-green11 shadow-[inset_0_0_0_1px] shadow-green7 hover:shadow-[inset_0_0_0_1px] hover:shadow-green8 focus:shadow-[0_0_0_2px] focus:shadow-green8"
        >
          Copy
        </button>
      </ToastAction>
    </ToastRoot>

    <ToastPortal :defer="true" to="body">
      <ToastViewport
        class="[--viewport-padding:_25px] fixed bottom-0 right-0 flex flex-col p-[var(--viewport-padding)] gap-[10px] w-[390px] max-w-[100vw] m-0 list-none z-[2147483647] outline-none"
      />
    </ToastPortal>
  </ToastProvider>
</template>

<style>
.toast-root-grid {
  grid-template-areas: 'title action' 'description action';
}
</style>
