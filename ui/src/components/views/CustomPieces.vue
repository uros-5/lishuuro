<script setup lang="ts">
import { createZipFile, extractZipFile, IndexedDBStorage } from '@/helpers/zip'
import router from '@/router'
import { onMounted, onUnmounted, ref } from 'vue'

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

enum Loading {
  ImportZip,
  OneByOne,
  Question,
}

const option = ref(Loading.Question)
const open = ref(false)

let fileNames = [
  'bA',
  'bB',
  'bC',
  'bG',
  'bK',
  'bN',
  'bP',
  'bQ',
  'bR',
  'wA',
  'wB',
  'wC',
  'wG',
  'wK',
  'wN',
  'wP',
  'wQ',
  'wR',
]
fileNames.forEach((value) => {
  if (!value.endsWith('.svg') || !value.endsWith('.png') || !value.endsWith('.jpg')) {
    fileNames.push(value + '.svg')
    fileNames.push(value + '.png')
    fileNames.push(value + '.jpg')
    fileNames.push(value + '.jpeg')
  }
})

const zipFile = ref({} as HTMLInputElement)
const shareZip = ref({} as HTMLAnchorElement)
const styleName = ref('')

function values(color: string) {
  return [
    { t: 'King', name: `${color}K`, value: undefined as undefined | File },
    { t: 'Queen', name: `${color}Q`, value: undefined as undefined | File },
    { t: 'Rook', name: `${color}R`, value: undefined as undefined | File },
    { t: 'Bishop', name: `${color}B`, value: undefined as undefined | File },
    { t: 'Knight', name: `${color}N`, value: undefined as undefined | File },
    { t: 'Pawn', name: `${color}P`, value: undefined as undefined | File },
    { t: 'Chancellor', name: `${color}C`, value: undefined as undefined | File },
    { t: 'ArchBishop', name: `${color}A`, value: undefined as undefined | File },
    { t: 'Giraffe', name: `${color}G`, value: undefined as undefined | File },
  ]
}

const players = ref([
  { title: 'Player 1', pieces: values('w') },
  { title: 'Player 2', pieces: values('b') },
])

const db = new IndexedDBStorage('lishuuro', 'zip')

onMounted(() => {
  document.body.dataset['page'] = 'custom-pieces'
})

onUnmounted(() => {})

async function addedZipFile() {
  if (!zipFile.value) return
  let firstItem = zipFile.value.files?.item(0)
  if (!firstItem) return
  let contents = await extractZipFile(firstItem)

  if (checkZip(contents)) {
    await db.setItem(firstItem.name, firstItem)
    router.push('/')
  }
}

function checkZip(contents: Record<string, Blob>): boolean {
  let valid = false
  Object.keys(contents).map((key) => {
    if (fileNames.includes(key)) {
      valid = true
    }
  })
  return valid
}

async function share(dl = false) {
  let style = styleName.value.length == 0 ? 'MyStyle' : styleName.value
  let canCreate = false
  let files: Record<string, Blob> = {}
  let acceptedExtensions = ['svg', 'png', 'jpg', 'jpeg']
  players.value.forEach((player) => {
    player.pieces.forEach((piece) => {
      if (piece.value != undefined) {
        let ext = piece.value.name.split('.').at(-1)
        if (!acceptedExtensions.includes(ext!)) return

        if (piece.value.name) files[piece.name + '.' + ext] = piece.value
        canCreate = true
      }
    })
  })
  if (canCreate) {
    let zip = await createZipFile(files)
    if (dl) {
      shareZip.value.href = URL.createObjectURL(zip)
      shareZip.value.download = style.replace('zip', '') + '.zip'
      shareZip.value.click()
    }
    return zip
  }
}

async function addToApp() {
  let style = styleName.value.length == 0 ? 'MyStyle' : styleName.value
  let file = await share()
  if (!file) return
  await db.setItem(style.replace('zip', '') + '.zip', file)
  open.value = true
}
</script>

<template>
  <section
    id="home"
    class="col-start-3 col-end-4 row-span-1 grid grid-cols-1 grid-rows-[auto_1fr] sm:grid-cols-[0.05fr_0.9fr_0.05fr] md:grid-cols-[0.05fr_0.3fr_0.8fr_0.05fr] py-5 gap-y-10 sm:gap-x-2 sm:gap-y-7 touch-auto"
  >
    <div
      class="row-start-1 row-end-2 col-span-1 sm:col-start-2 sm:col-end-4 sm:text-xl md:text-2xl font-main dark:text-main-200 text-center"
    >
      Create custom piece set
    </div>

    <div
      v-if="option == Loading.Question"
      class="row-start-2 row-end-3 col-span-1 sm:col-start-2 sm:col-end-4 flex p-5 gap-3 md:gap-8 md:text-2xl justify-center"
    >
      <button
        @click="zipFile.click()"
        class="border rounded-sm p-3 text-main-800 dark:text-main-300 font-sec shadow-sm shadow-sec-500 dark:shadow-sec-100 bg-gradient-to-br from-main-300 dark:from-main-800 to-main-200/70 dark:to-main-950/70 cursor-pointer transition duration-400 hover:to-main-50 hover:dark:to-main-700"
      >
        Load from zip
      </button>
      <button
        @click="option = Loading.OneByOne"
        class="border rounded-sm p-3 text-main-800 dark:text-main-300 font-sec shadow-sm shadow-sec-500 dark:shadow-sec-100 bg-gradient-to-br from-main-300 dark:from-main-800 to-main-200/70 dark:to-main-950/70 cursor-pointer transition duration-400 hover:to-main-50 hover:dark:to-main-700"
      >
        Create piece set
      </button>
      <input
        ref="zipFile"
        class="hidden"
        type="file"
        accept=".zip"
        id="upload-file"
        @change="(e) => addedZipFile()"
      />
    </div>

    <div
      v-if="option == Loading.OneByOne"
      class="row-start-2 row-end-3 col-span-1 sm:col-start-2 sm:col-end-4 sm:text-xl md:text-2xl font-main dark:text-main-200 text-center flex flex-col gap-y-9 gap-x-7 sm:flex-row justify-center"
    >
      <input
        type="text"
        class="border-2 border-b-main-800 dark:border-b-main-100 p-1 shadow-sm shadow-main-200 dark:shadow-main-500"
        v-model="styleName"
        placeholder="Name of style"
      />
    </div>

    <div
      v-if="option == Loading.OneByOne"
      class="row-start-3 row-end-4 col-span-1 sm:col-start-2 sm:col-end-4 flex flex-col sm:flex-row gap-15 justify-center"
    >
      <div
        class="sm:text-xl md:text-2xl font-main dark:text-main-200 text-center flex flex-col gap-y-9"
        v-for="player in players"
      >
        <h1 class="underline underline-offset-4">{{ player.title }}</h1>
        <div class="flex flex-col items-center gap-y-3" v-for="piece in player.pieces">
          <p>{{ piece.t }}</p>

          <div class="relative">
            <label
              title="Click to upload"
              :for="player.title + piece.t"
              class="cursor-pointer flex items-center gap-4 px-6 py-4 hover:before:border-gray-300 group before:absolute before:inset-0 before:rounded-3xl before:border-dashed before:transition-transform before:duration-300 hover:before:scale-105 active:duration-75 active:before:scale-95 before:bg-gradient-to-r before:from-main-300 dark:before:from-main-600/50 before:dark:to-main-300/70 dark:before:to-main-500/70"
            >
              <div class="relative">
                <span
                  class="block text-base font-semibold relative text-main-700 dark:text-main-100 dark:group-hover:text-main-300 group-hover:text-main-600"
                >
                  Upload a file
                </span>
                <span class="mt-0.5 block text-sm md:text-2xl dark:text-gray-300 text-gray-800"
                  >Max 2 MB</span
                >
                <span>{{ piece.value ? piece.value.name : '' }}</span>
              </div>
            </label>
            <input
              hidden=""
              type="file"
              @change="
                (ev) => {
                  let event = ev as InputEvent
                  let target = event.target as HTMLInputElement
                  if (!target) return
                  if (!target.files) return
                  if (!target.files.item(0)) return
                  piece.value = target.files.item(0) as File
                }
              "
              :id="player.title + piece.t"
              accept="image/*"
            />
          </div>
        </div>
      </div>
    </div>
    <div
      v-if="option == Loading.OneByOne"
      class="row-start-4 row-end-5 col-span-1 sm:col-start-2 sm:col-end-4 sm:text-xl md:text-2xl font-main dark:text-main-200 text-center flex flex-col gap-y-9 gap-x-7 sm:flex-row justify-center"
    >
      <button
        @click="addToApp()"
        class="appearance-none border-2 p-2 rounded dark:bg-main-700 dark:text-main-50 cursor-pointer transition-transform hover:scale-110 active:bg-main-50"
      >
        Add to app
      </button>
      <button
        @click="share(true)"
        class="appearance-none border-2 p-2 rounded dark:bg-main-700 dark:text-main-50 cursor-pointer transition-transform hover:scale-110 active:bg-main-50"
      >
        Share style
      </button>
      <a ref="shareZip" href="/" class="hidden"></a>
    </div>
  </section>

  <ToastProvider>
    <ToastRoot
      v-model:open="open"
      class="bg-white toast-root-grid rounded-lg shadow-sm border p-[15px] grid grid-cols-[auto_max-content] gap-x-[15px] items-center data-[state=open]:animate-slideIn data-[state=closed]:animate-hide data-[swipe=move]:translate-x-[var(--reka-toast-swipe-move-x)] data-[swipe=cancel]:translate-x-0 data-[swipe=cancel]:transition-[transform_200ms_ease-out] data-[swipe=end]:animate-swipeOut"
    >
      <ToastTitle class="[grid-area:_title] mb-[5px] font-medium text-slate12 text-sm">
        Piece set
      </ToastTitle>
      <ToastDescription as-child>
        <p class="[grid-area:_description] m-0 text-slate11 text-xs leading-[1.3]">
          {{ styleName.length == 0 ? 'MyStyle' : styleName }}
        </p>
        <!-- <p class="!select-text">{{ frontend() }}game/{{ store.state._id }}</p> -->
      </ToastDescription>
      <ToastAction class="[grid-area:_action]" as-child alt-text="Copy game url">
        <button
          @click="open = false"
          class="inline-flex items-center justify-center rounded-md font-medium text-xs px-[10px] leading-[25px] h-[25px] bg-green2 text-green11 shadow-[inset_0_0_0_1px] shadow-green7 hover:shadow-[inset_0_0_0_1px] hover:shadow-green8 focus:shadow-[0_0_0_2px] focus:shadow-green8"
        >
          Close
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
