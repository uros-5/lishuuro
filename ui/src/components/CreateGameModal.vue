<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui'
import { ref } from 'vue'

const props = defineProps<{ requestType: 'vsFriend' | 'vsAi' }>()

const emits = defineEmits(['closeModal', 'saveForm'])

const color = ref(2 as 0 | 1 | 2)
const level = ref(1)
const name = ref('')

function send() {
  if (props.requestType == 'vsFriend') {
    emits('saveForm', { selected: true, name: name.value }, color.value)
  } else if (props.requestType == 'vsAi') {
    emits('saveForm', { selected: true, level: level.value }, color.value)
  }
}
</script>

<template>
  <DialogRoot
    :defaultOpen="true"
    @update:open="
      (v) => {
        if (v == false) {
          $emit('closeModal')
        }
      }
    "
  >
    <DialogPortal>
      <DialogOverlay
        class="bg-main-900/80 dark:bg-main-700/90 data-[state=open]:animate-overlayShow fixed inset-0 z-30"
      />
      <DialogContent
        class="font-main text-xl bg-main-100 dark:bg-main-800 dark:text-main-50 data-[state=open]:animate-contentShow fixed top-[50%] left-[50%] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] p-[25px] shadow-[hsl(206_22%_7%_/_35%)_0px_10px_38px_-10px,_hsl(206_22%_7%_/_20%)_0px_10px_20px_-15px] focus:outline-none z-[100]"
      >
        <DialogTitle class="text-mauve12 m-0 text-[17px] font-semibold"
          >Send game request{{
            props.requestType == 'vsFriend' ? ' (friend)' : ' (AI)'
          }}</DialogTitle
        >
        <DialogDescription class="text-mauve11 mt-[10px] mb-5 text-sm leading-normal">
        </DialogDescription>

        <div class="w-full flex flex-col md:text-2xl">
          <div
            class="text-center row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5 my-5"
          >
            Play as:
          </div>

          <div
            class="flex justify-center gap-3 md:gap-5 md:h-auto row-span-1 col-start-2 col-end-2 md:col-start-2 md:col-end-5"
          >
            <button
              @click="color = 1"
              class="mx-0 my-0 w-[50px] h-[50px] md:w-[64px] md:h-[64px] bg-main-300 dark:bg-main-900 shadow shadow-main-200 dark:shadow-main-600 hover:bg-main-100/50 dark:hover:bg-main-700/50 hover:cursor-pointer transition-[background7 duration-200"
              :class="{ ' bg-main-100/50! dark:bg-main-700/50! ': color == 1 }"
            >
              <i
                class="block w-[50px] h-[50px] md:w-[64px] md:h-[64px] [background-size:50px_50px] md:[background-size:64px_64px] [background-image:url(/pieces/merida/bK.svg)]"
              ></i>
            </button>
            <button
              @click="color = 2"
              class="mx-0 -my-2 w-[60px] h-[60px] md:w-[80px] md:h-[80px] bg-main-300 dark:bg-main-900 shadow shadow-main-200 dark:shadow-main-600 hover:bg-main-100/50 dark:hover:bg-main-700/50 hover:cursor-pointer transition-[background7 duration-200"
              :class="{ ' bg-main-100/50! dark:bg-main-700/50! ': color == 2 }"
            >
              <i
                class="block w-[60px] h-[60px] md:w-[80px] md:h-[80px] [background-size:60px_60px] md:[background-size:80px_80px] [background-image:url(/pieces/merida/wbK.svg)]"
              ></i>
            </button>
            <button
              @click="color = 0"
              class="mx-0.5 my-0 w-[50px] h-[50px] md:w-[64px] md:h-[64px] bg-main-300 dark:bg-main-900 shadow shadow-main-200 dark:shadow-main-600 hover:bg-main-100/50 dark:hover:bg-main-700/50 hover:cursor-pointer transition-[background7 duration-200"
              :class="{ ' bg-main-100/50! dark:bg-main-700/50! ': color == 0 }"
            >
              <i
                class="block w-[50px] h-[50px] md:w-[64px] md:h-[64px] [background-size:50px_50px] md:[background-size:64px_64px] [background-image:url(/public/pieces/merida/wK.svg)]"
              ></i>
            </button>
          </div>
        </div>

        <div
          class="mt-[25px] flex justify-end gap-3 items-center"
          :class="{ ' flex-wrap ': props.requestType == 'vsAi' }"
        >
          <input
            v-model="name"
            v-if="props.requestType == 'vsFriend'"
            class="shadow shadow-main-200 dark:shadow-main-600 p-2 focus:outline focus:outline-sky-300 focus:dark:outline-main-200"
            type="text"
            placeholder="Enter player name(optional)"
          />

          <div v-if="props.requestType == 'vsAi'" class="text-center">Level {{ level }}</div>

          <input
            v-model.number="level"
            class="slider cursor-pointer"
            v-if="props.requestType == 'vsAi'"
            type="range"
            min="1"
            max="4"
            id="aiLevel"
          />
          <DialogClose as-child @click="send()">
            <button
              class="border-2 text-sm hover:bg-gray-300 hover:dark:bg-gray-800 hover:cursor-pointer focus:shadow-green7 inline-flex h-[35px] items-center justify-center rounded-lg px-[15px] font-semibold leading-none focus:shadow-[0_0_0_2px] focus:outline-none p-5"
            >
              <span v-if="props.requestType == 'vsFriend'"> Create url </span
              ><span v-if="props.requestType == 'vsAi'"> Start </span>
            </button>
          </DialogClose>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style></style>
