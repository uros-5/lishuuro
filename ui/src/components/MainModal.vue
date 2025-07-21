<script setup lang="ts">
import { ntw } from '@/not-tailwind'
import { usePieces } from '@/stores/game/usePieces'
import { useGameSettings } from '@/stores/gameSettings'
import { useModal } from '@/stores/modal'
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui'
import { onMounted, onUnmounted } from 'vue'

const modal = useModal()

const settings = useGameSettings()
let piecesStore = usePieces()

const lightBorder = ntw.get('--color-main-600')
const darkBorder = ntw.get('--color-main-200')
onMounted(() => {
  document.body.style.setProperty('--lightBorder', `var(${lightBorder})`)
  document.body.style.setProperty('--darkBorder', `var(${darkBorder})`)
})

onMounted(async () => {
  await piecesStore.loadPieces()
})

onUnmounted(() => {
  document.body.style.removeProperty('--lightBorder')
  document.body.style.removeProperty('--darkBorder')
})
const boards = [
  { value: 'blue', url: '/board/8x8blue.svg' },
  { value: 'brown', url: '/board/8x8brown.svg' },
  { value: 'brown_sand', url: '/board/8x8brown_sand.svg' },
  { value: 'brown_yellow', url: '/board/8x8brown_yellow.svg' },
  { value: 'gray', url: '/board/8x8gray.svg' },
  { value: 'green', url: '/board/8x8green.svg' },
]
</script>

<template>
  <DialogRoot
    :defaultOpen="modal.active == 'game-settings'"
    @update:open="
      (v) => {
        if (v == false) {
          modal.toggle()
        }
      }
    "
    v-if="modal.active == 'game-settings'"
  >
    <DialogPortal>
      <DialogOverlay
        class="bg-main-900/80 dark:bg-main-700/90 data-[state=open]:animate-overlayShow fixed inset-0 z-30"
      />
      <DialogContent
        class="font-main text-xl bg-main-100 dark:bg-main-800 dark:text-main-50 data-[state=open]:animate-contentShow fixed top-[50%] left-[50%] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] p-[25px] shadow-[hsl(206_22%_7%_/_35%)_0px_10px_38px_-10px,_hsl(206_22%_7%_/_20%)_0px_10px_20px_-15px] focus:outline-none z-[100]"
      >
        <DialogTitle class="font-sec-bold text-mauve12 m-0 text-[17px] font-semibold">
          Game settings
        </DialogTitle>
        <fieldset class="mb-[15px] flex items-center justify-around gap-5">
          <label for="sound">Sound</label>
          <input id="sound" class="w-[100%] p-3 slider sound" min="1" max="100" type="range" />
        </fieldset>
        <div class="mb-[15px] flex items-center justify-around gap-5">
          <p>Board</p>

          <fieldset class="settings grid grid-cols-[repeat(3,auto)] justify-around gap-3 p-2">
            <template v-for="i in boards">
              <input
                checked
                @change="settings.updateBoard(i.value)"
                class="absolute scale-0 opacity-0"
                name="board"
                :id="`board${i.value}`"
                :value="i.value"
                type="radio"
              />
              <label
                :for="`board${i.value}`"
                :style="`background-image: url(${i.url});`"
                :data-b="`background-image: url(${i.url});`"
                class="[background-size:95%_95%] bg-center mx-auto bg-no-repeat min-w-0 p-8 hover:shadow-md hover:shadow-main-800 dark:hover:shadow-main-300 hover:cursor-pointer checked:border-2 checked:bg-main-800 cursor-pointer"
              ></label>
            </template>
          </fieldset>
        </div>

        <div class="mb-[15px] flex items-center justify-around gap-5">
          <p>Pieces</p>

          <fieldset class="settings grid grid-cols-3 gap-3 p-2">
            <template v-for="i in piecesStore.pieces">
              <input
                @change="settings.updatePiece(i.value)"
                class="absolute scale-0 opacity-0"
                type="radio"
                name="piece"
                :id="`piece${i.value}`"
                :value="i.value"
              />
              <label
                :title="i.value"
                class="bg-center bg-no-repeat p-8 [background-size:90%] hover:shadow-md hover:shadow-main-800 dark:hover:shadow-main-300 hover:cursor-pointer"
                :for="`piece${i.value}`"
                :style="`background-image: url(${i.url});`"
              ></label>
            </template>
          </fieldset>
        </div>

        <DialogClose
          class="text-grass11 hover:bg-green4 focus:shadow-green7 absolute top-[10px] right-[10px] inline-flex h-[25px] w-[25px] appearance-none items-center justify-center rounded-full focus:shadow-[0_0_0_2px] focus:outline-none"
          aria-label="Close"
        >
        </DialogClose>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style>
.settings label {
  transition: border 0.2s ease;
}

.settings input[type='radio']:checked + label {
  border: 2px solid var(--lightBorder);
}

@media (prefers-color-scheme: dark) {
  .settings input[type='radio']:checked + label {
    border: 2px solid var(--darkBorder);
  }
}
</style>
